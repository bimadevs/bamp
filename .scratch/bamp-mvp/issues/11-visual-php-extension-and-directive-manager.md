# 11 - Visual PHP Extension & Directive Manager

Status: resolved

## Goal
Implement a graphical PHP Configuration manager allowing developers to toggle PHP extensions and adjust critical directives (`memory_limit`, `upload_max_filesize`, etc.) via UI switches that modify `php.ini` and reload PHP-FPM.

## Requirements
1. Backend `php.ini` manipulation:
   - Identify active `php.ini` location.
   - Parse available and active extensions (`xdebug`, `pdo_mysql`, `intl`, `gd`, `zip`, `redis`, `opcache`, `curl`, `mbstring`).
   - Read and write core directives (`memory_limit`, `upload_max_filesize`, `post_max_size`, `max_execution_time`, `date.timezone`).
   - Gracefully reload PHP-FPM after modification.
   - Expose Tauri commands: `get_php_config`, `toggle_php_extension`, `update_php_directive`.
2. Frontend modal:
   - Modern tabbed dialog with switch toggles and input fields.

## Completion Criteria
- [x] Backend parses extensions and directives from `php.ini`.
- [x] Toggling an extension uncomments or comments the line in `php.ini`.
- [x] Updating a directive updates the value in `php.ini`.
- [x] PHP-FPM reloads cleanly upon configuration save.
- [x] Unit test `test_php_ini_manipulation` passes.

## Comments
- Implemented `PhpManager` in `src-tauri/src/php/mod.rs`.
- Created interactive modal dialog in `App.tsx` with Extensions and Directives tabs.
- Added IPC commands `get_php_config`, `toggle_php_extension`, and `update_php_directive`.
