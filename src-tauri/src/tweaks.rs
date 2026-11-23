use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, io, path::PathBuf};
use winreg::{enums::{HKEY_CURRENT_USER, REG_DWORD}, RegKey};

#[derive(Clone, Copy)]
struct Target { path: &'static str, name: &'static str }
const CAPTURE: &[Target] = &[
    Target { path: r"Software\Microsoft\Windows\CurrentVersion\GameDVR", name: "AppCaptureEnabled" },
    Target { path: r"System\GameConfigStore", name: "GameDVR_Enabled" },
];
const TRANSPARENCY: &[Target] = &[Target { path: r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize", name: "EnableTransparency" }];
const SUGGESTIONS: &[Target] = &[Target { path: r"Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager", name: "SubscribedContent-338388Enabled" }];
fn targets(id: &str) -> Result<&'static [Target], String> {
    match id { "capture" => Ok(CAPTURE), "transparency" => Ok(TRANSPARENCY), "suggestions" => Ok(SUGGESTIONS), _ => Err("Tweak não autorizado".into()) }
}
#[derive(Serialize, Deserialize, Clone)]
struct Original { existed: bool, value: u32 }
type Backup = BTreeMap<String, Original>;
fn backup_path() -> Result<PathBuf, String> {
    let local = std::env::var_os("LOCALAPPDATA").ok_or("LOCALAPPDATA indisponível")?;
    let dir = PathBuf::from(local).join("Veyra");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("registry-backup-v1.json"))
}
fn load() -> Result<Backup, String> {
    let path = backup_path()?;
    if !path.exists() { return Ok(Backup::new()); }
    serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?).map_err(|e| format!("Backup inválido: {e}"))
}
fn save(data: &Backup) -> Result<(), String> {
    let path = backup_path()?;
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, serde_json::to_vec_pretty(data).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    // Windows rename cannot replace an existing file; retain the previous backup until new bytes are ready.
    if path.exists() {
        let old = path.with_extension("previous");
        if old.exists() { fs::remove_file(&old).map_err(|e| e.to_string())?; }
        fs::rename(&path, &old).map_err(|e| e.to_string())?;
        if let Err(e) = fs::rename(&tmp, &path) { let _ = fs::rename(&old, &path); return Err(e.to_string()); }
        let _ = fs::remove_file(old);
    } else { fs::rename(tmp, path).map_err(|e| e.to_string())?; }
    Ok(())
}
fn key(t: Target) -> String { format!("{}|{}", t.path, t.name) }
fn read(t: Target) -> Result<Option<u32>, String> {
    let root = RegKey::predef(HKEY_CURRENT_USER);
    let k = match root.open_subkey(t.path) { Ok(k) => k, Err(e) if e.kind()==io::ErrorKind::NotFound => return Ok(None), Err(e) => return Err(e.to_string()) };
    match k.get_raw_value(t.name) {
        Ok(v) if v.vtype == REG_DWORD && v.bytes.len()==4 => Ok(Some(u32::from_le_bytes(v.bytes[..4].try_into().map_err(|_| "DWORD inválido")?))),
        Ok(_) => Err(format!("Tipo inesperado: {}", t.name)),
        Err(e) if e.kind()==io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}
fn write(t: Target, val: Option<u32>) -> Result<(), String> {
    let root = RegKey::predef(HKEY_CURRENT_USER);
    let (k, _) = root.create_subkey(t.path).map_err(|e| e.to_string())?;
    match val { Some(n) => k.set_value(t.name, &n).map_err(|e| e.to_string()), None => match k.delete_value(t.name) { Ok(()) => Ok(()), Err(e) if e.kind()==io::ErrorKind::NotFound => Ok(()), Err(e) => Err(e.to_string()) } }
}
#[derive(Serialize)]
pub struct TweakStatus { id: String, status: String, can_apply: bool, can_restore: bool }
fn status(id: &str, backup: &Backup) -> Result<TweakStatus, String> {
    let ts = targets(id)?;
    let saved = ts.iter().filter(|t| backup.contains_key(&key(**t))).count();
    let values = ts.iter().map(|t| read(*t)).collect::<Result<Vec<_>,_>>()?;
    let applied = values.iter().all(|v| *v==Some(0));
    let state = if saved>0 && saved<ts.len() { "Backup parcial — bloqueado" }
        else if saved==ts.len() && applied { "Aplicado · backup disponível" }
        else if saved==ts.len() { "Alterado externamente · backup disponível" }
        else if applied { "Já configurado externamente" }
        else { "Não aplicado" };
    Ok(TweakStatus { id:id.into(), status:state.into(), can_apply:saved==0 && !applied, can_restore:saved==ts.len() && applied })
}
#[tauri::command]
pub fn tweak_statuses() -> Result<Vec<TweakStatus>, String> {
    let b=load()?;
    ["capture","transparency","suggestions"].iter().map(|id| status(id,&b)).collect()
}
#[tauri::command]
pub fn apply_tweak(id: String) -> Result<(), String> {
    let ts=targets(&id)?;
    let mut b=load()?;
    let before=ts.iter().map(|t| read(*t)).collect::<Result<Vec<_>,_>>()?;
    if ts.iter().any(|t| b.contains_key(&key(*t))) { return Err("Backup já existe. Restaure antes de reaplicar.".into()); }
    if before.iter().all(|v| *v==Some(0)) { return Err("Já configurado externamente; nenhuma alteração necessária".into()); }
    for (t,v) in ts.iter().zip(before.iter()) { b.insert(key(*t), Original { existed:v.is_some(), value:v.unwrap_or(0) }); }
    save(&b)?; // backup persisted before any registry modification
    for t in ts { write(*t,Some(0))?; }
    Ok(())
}
#[tauri::command]
pub fn restore_tweak(id: String) -> Result<(), String> {
    let ts=targets(&id)?;
    let mut b=load()?;
    if ts.iter().any(|t| !b.contains_key(&key(*t))) { return Err("Backup completo não encontrado".into()); }
    if ts.iter().map(|t| read(*t)).collect::<Result<Vec<_>,_>>()?.iter().any(|v| *v!=Some(0)) { return Err("Alteração externa detectada; restauração bloqueada".into()); }
    for t in ts { let old=b.get(&key(*t)).ok_or("Backup ausente")?; write(*t,if old.existed {Some(old.value)} else {None})?; }
    for t in ts { b.remove(&key(*t)); }
    save(&b)?;
    Ok(())
}
#[derive(Serialize)]
pub struct SystemInfo { processor: String, memory_gb: String, windows: String, storage: String }
#[tauri::command]
pub fn system_info() -> Result<SystemInfo, String> {
    use std::process::Command;
    // Windows built-in PowerShell only reads system information; no scripts from external sources.
    let ps = r#"$cpu=(Get-CimInstance Win32_Processor | Select-Object -First 1 -ExpandProperty Name); $ram=[math]::Round((Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory/1GB,1); $os=(Get-CimInstance Win32_OperatingSystem).Caption; $disk=Get-CimInstance Win32_LogicalDisk -Filter "DeviceID='C:'"; [pscustomobject]@{processor=$cpu;memory_gb="$ram GB";windows=$os;storage=if($disk){"$([math]::Round($disk.FreeSpace/1GB,1)) GB livres"}else{'Indisponível'}} | ConvertTo-Json -Compress"#;
    let out=Command::new("powershell.exe").args(["-NoProfile","-NonInteractive","-Command",ps]).output().map_err(|e|e.to_string())?;
    if !out.status.success() { return Err("Não foi possível ler as informações do sistema".into()); }
    serde_json::from_slice::<serde_json::Value>(&out.stdout).map(|v| SystemInfo {
        processor:v["processor"].as_str().unwrap_or("Indisponível").into(),
        memory_gb:v["memory_gb"].as_str().unwrap_or("Indisponível").into(),
        windows:v["windows"].as_str().unwrap_or("Indisponível").into(),
        storage:v["storage"].as_str().unwrap_or("Indisponível").into(),
    }).map_err(|e|e.to_string())
}
