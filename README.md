# Veyra v0.8 — Functional Foundation

Windows desktop app built with React 19, TypeScript, Tauri 2 and Rust.

## New in v0.8
- Reads real CPU, installed RAM, Windows name, and free C: disk space through built-in Windows CIM.
- Three allowlisted, per-user registry tweaks: background capture (two DWORDs), transparency, and one content suggestion setting.
- Read-only status checks, explicit confirmation, durable per-value backup before modification, guarded restore and disabled buttons when inappropriate.
- Advanced/legacy search tweak remains unavailable. No administrator privileges required for the intended HKCU operations.

## Safety and limitations
- Backups live in `%LOCALAPPDATA%\\Veyra\\registry-backup-v1.json`; they are **not** the old MATSU backup format and no migration is performed. Keep old backups and app until you verify their restorations.
- If the registry changes externally after Veyra applies a tweak, restoration is blocked rather than overwriting external changes.
- If an operation fails halfway, the backup remains available; verify state and do not delete it. This is a first functional preview, not an audited optimizer.
- Hardware detection may fail depending on Windows permissions and PowerShell availability; failures are shown rather than replaced with fake data.
- Only test after reviewing the source and backing up important data. These settings may impact game recording or require sign-out.

## Build
Use the included `.github/workflows/build-windows.yml` on the `main` branch, or on Windows install Node.js 22, Rust/MSVC toolchain, and run `npm install` then `npm run tauri build`.
