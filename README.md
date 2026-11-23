# Veyra v0.7 — React + Tauri

Visual preview of the Windows optimization utility formerly called MATSU Optimizer.

## What's new
- Rebranded product, app metadata, Windows installer and icons to **Veyra**.
- Custom frameless title bar with native minimize/maximize/close controls.
- Fixed sidebar/header; scrolling limited to main content.
- Original animations and visual layout retained.

## Important
This is a **visual preview only**. Tweaks do not change Windows and hardware information is illustrative. Do not remove your previous WPF installation or backups.

## Build via GitHub
Upload extracted project contents to a **separate repository** or replace files in your React preview repository. Commit to `main` and check Actions > Build Veyra Windows Preview. Download the NSIS installer from Artifacts.

## Local
`npm install` then `npm run tauri dev` (requires Rust and Windows WebView2); `npm run tauri build` creates installer.

Note: Veyra is a provisional brand name; trademark/domain availability has not been checked.
