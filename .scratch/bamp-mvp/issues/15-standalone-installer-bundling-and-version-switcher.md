# 15 - Standalone Installer Bundling & Version Switcher

Status: resolved

## Goal
Bundle all default service binaries (Nginx, Apache, PHP 8.3, MariaDB, Mailpit) directly into the Windows release installer (.exe) via CI, provide a robust version-specific installation script (`scripts/install-service-version.ps1`) for downloading other versions (PHP 7.4, 8.1, 8.2, 8.4), and integrate them with BAMP's Package Hub.

## Requirements
1. **Version-Specific Installation Script (`scripts/install-service-version.ps1`)**:
   - Accepts parameters: `-Service <name>`, `-Version <ver>`, and convenience switch `-Php <7.4|8.1|8.2|8.3|8.4>`.
   - Downloads and sets up specified version into `bin/php/<version>/` with configured `php.ini`.
2. **Tauri Resource Bundling (`src-tauri/tauri.conf.json`)**:
   - Configure `"bundle.resources"` to package `bin/`, `templates/`, `data/adminer/`, and `scripts/` into the distribution package.
3. **Robust Base Directory Resolution (`src-tauri/src/lib.rs`)**:
   - Resolve `base_dir` relative to `current_exe()` if `current_dir()` does not contain `bin/` or `templates/` (vital for Start Menu / Desktop shortcuts on Windows).
4. **Package Hub Integration (`src-tauri/src/tools/mod.rs`)**:
   - Update `install_package` to invoke `scripts/install-service-version.ps1` for on-demand PHP version downloads.
5. **CI Pre-Bundling in GitHub Actions (`.github/workflows/release.yml`)**:
   - In `build-windows`, run `install-binaries-win.ps1` before `pnpm tauri build` so the generated Windows installer (`.exe` / `.msi`) and portable zip contain all default binaries out-of-the-box.

## Completion Criteria
- [x] `scripts/install-service-version.ps1` written, tested, and documented.
- [x] `tauri.conf.json` defines bundle resources for `bin/`, `templates/`, `data/adminer/`, `scripts/`.
- [x] `base_dir` resolution handles executable path alongside current working directory.
- [x] `install_package` invokes `install-service-version.ps1` when installing modular PHP packages.
- [x] `.github/workflows/release.yml` pre-provisions binaries on Windows runners and packages both installer and portable zip.
- [x] `cargo check`, `cargo test`, and `pnpm build` pass.

## Comments
- Implemented `scripts/install-service-version.ps1` with dedicated download mappings for PHP 8.4, 8.3, 8.2, 8.1, 7.4, MariaDB, Apache, Nginx, and Mailpit.
- Configured Tauri v2 resource bundling in `src-tauri/tauri.conf.json` mapping `../bin`, `../templates`, `../data/adminer`, and `../scripts`.
- Added `resolve_bamp_base_dir()` in `src-tauri/src/lib.rs` checking both `current_dir()` and `current_exe().parent()` so installed desktop shortcuts locate assets properly.
- Enhanced `ToolsManager::install_package` to invoke `scripts/install-service-version.ps1` via PowerShell when available.
- Updated `.github/workflows/release.yml` to download all Windows binaries during CI on `windows-latest` prior to packaging, generating pre-bundled NSIS installers and a pre-bundled standalone portable zip package (`bamp-windows-x86_64-portable.zip`).
