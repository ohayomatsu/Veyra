#[cfg(windows)]
mod tweaks;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default();
    #[cfg(windows)]
    let builder = builder.invoke_handler(tauri::generate_handler![tweaks::system_info, tweaks::tweak_statuses, tweaks::apply_tweak, tweaks::restore_tweak]);
    builder.run(tauri::generate_context!()).expect("error while running Veyra");
}
