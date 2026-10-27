# 14 - Windows Portable Binaries & Service Runner

Status: resolved

## Goal
Provide full support for downloading, extracting, and executing Windows service binaries (Nginx, Apache HTTPD, PHP 8.3 NTS, MariaDB, Mailpit) into `bin/`, including automated setup scripts, Rust backend Windows FastCGI adaptations, and UI integration.

## Requirements
1. **Automated Windows Binary Downloader Script (`scripts/install-binaries-win.ps1`)**:
   - Downloads official portable Windows binaries:
     - Nginx (nginx.org Windows zip) -> `bin/nginx/`
     - Apache HTTPD (Apache Lounge Win64 zip) -> `bin/apache/`
     - PHP 8.3 NTS x64 (windows.php.net zip) -> `bin/php/php-8.3/`
     - MariaDB 10.11 LTS Win64 (mariadb.org zip) -> `bin/mysql/`
     - Mailpit (axllent/mailpit windows-amd64 zip) -> `bin/mailpit/`
   - Unpacks archives into designated `bin/` directories with correct folder structures.
   - Initializes MariaDB data directory if needed.
2. **Repository Hygiene**:
   - Update `.gitignore` so large downloaded binary files in `bin/` are excluded from Git, preserving BAMP's <20MB core requirement (ADR-0001).
3. **Rust Backend Supervisor & Platform Resolution**:
   - Update `resolve_binary` in `src-tauri/src/platform/mod.rs` to detect binaries in subdirectories (e.g. `bin/mysql/bin/mariadbd.exe`, `bin/apache/bin/httpd.exe`, `bin/php/php-8.3/php-cgi.exe`).
   - Update service definitions in `src-tauri/src/lib.rs` and `src-tauri/src/supervisor/mod.rs`:
     - On Windows, register `php-cgi` / `php-cgi.exe` as primary candidate for PHP, spawning FastCGI with `-b 127.0.0.1:<port>` and `PHP_FCGI_MAX_REQUESTS=0`.
     - On Windows, configure MariaDB/MySQL with `--console` and appropriate defaults.
4. **Backend Command & Frontend Trigger**:
   - Expose `download_service_binaries` Tauri IPC command in `src-tauri/src/lib.rs` / `tools/mod.rs`.
   - Add "Download Binaries" trigger in UI when service binaries are missing.

## Completion Criteria
- [x] `scripts/install-binaries-win.ps1` exists, documented, and executable.
- [x] `.gitignore` ignores binary directories in `bin/` while keeping directory placeholders.
- [x] `resolve_binary` finds nested binaries in `bin/<service>/bin/` and `bin/php/<version>/`.
- [x] Supervisor seamlessly launches `php-cgi.exe` with FastCGI on Windows.
- [x] `cargo check` and `cargo test` pass (14 unit tests passing).
- [x] `pnpm build` passes.

## Comments
- Created `scripts/install-binaries-win.ps1` supporting automated downloading and extraction of Nginx, Apache HTTPD, PHP 8.3, MariaDB, and Mailpit with custom `-Services` and `-Force` options.
- Added documentation in `scripts/README.md`.
- Updated `.gitignore` to prevent bloating Git with extracted binaries while committing placeholder `.gitkeep` files in `bin/nginx/`, `bin/apache/`, `bin/mysql/`, `bin/php/`.
- Enhanced `resolve_binary` in `src-tauri/src/platform/mod.rs` with recursive subdirectory scanning for versioned PHP folders (`bin/php/php-8.3/`) and nested `bin/` folders (`bin/mysql/bin/`, `bin/apache/bin/`), verified with unit test `test_resolve_binary_nested_and_versioned`.
- Adapted `src-tauri/src/supervisor/mod.rs` to automatically bind `php-cgi` to FastCGI on `127.0.0.1:<port>` with `PHP_FCGI_MAX_REQUESTS=0` and `-c data/php/php.ini`.
- Added `--console` flag for MySQL/MariaDB on Windows in `src-tauri/src/lib.rs`.
- Exposed `download_service_binaries` Tauri command and integrated UI triggers in both the Service error cards and Modular Package Hub modal.
