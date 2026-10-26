# BAMP: Product Requirements Document (PRD)

## 1. Product Vision

Deliver the fastest, lightest, and most developer-friendly local development stack manager for the modern PHP ecosystem. BAMP matches and exceeds the convenience of Laragon and XAMPP while maintaining a sub-30MB memory footprint, rock-solid process supervision, and full cross-platform compatibility (Windows and Linux).

## 2. Target Personas

- **Full-Stack PHP & WordPress Developers**: Need instant project bootstrapping, multi-version PHP support, fast database management, and valid HTTPS certificates.
- **Freelancers & Agency Engineers**: Concurrently juggle legacy PHP 7.4 projects and modern PHP 8.3/8.4 Laravel apps on the same machine without port or version clashes.
- **Developers on Resource-Constrained Machines**: Require development tools that do not consume gigabytes of RAM (avoiding Docker and Electron).

## 3. Functional Requirements

### 3.1 Service Management (The Stack)
- **FR-1.1**: One-click **Start All**, **Stop All**, and **Restart All** from Dashboard and System Tray.
- **FR-1.2**: Independent lifecycle control for each individual service: Nginx, Apache, PHP-FPM, MySQL/MariaDB, and Mailpit.
- **FR-1.3**: Dual web server support: Nginx as the primary lightweight default, with one-click switch to Apache for `.htaccess` compatibility.
- **FR-1.4**: Port Conflict Inspector: Detect occupied ports (80, 443, 3306, 8025, 1025) before starting a service, identifying conflicting PID and process name.
- **FR-1.5**: Anti-zombie process termination: All spawned child processes terminate unconditionally when BAMP closes.

### 3.2 Dynamic Virtual Hosts & Routing
- **FR-2.1**: Automated discovery of project folders inside default root `www/` and registered Linked Folders.
- **FR-2.2**: Automatic local domain resolution: Map `<folder-name>` to `http://<folder-name>.test` and `https://<folder-name>.test`.
- **FR-2.3**: Embedded UDP DNS server on port 53 returning `127.0.0.1` for all `*.test` queries, with automatic fallback to `/etc/hosts` or Windows `hosts` modification if port 53 is occupied.
- **FR-2.4**: Per-Project PHP Version Isolation: Allow individual virtual hosts to specify an explicit PHP version override (e.g. PHP 7.4) while other sites use the global default (e.g. PHP 8.3).
- **FR-2.5**: One-click actions per site: Open in Browser, Open in VS Code / IDE, Open in File Explorer.

### 3.3 Security & Local SSL
- **FR-3.1**: Automated Local Root CA generation using standard x509 standards.
- **FR-3.2**: One-click Root CA installation into system and browser trust stores (Windows CryptoAPI, Linux ca-certificates, NSS stores).
- **FR-3.3**: Dynamic generation and renewal of trusted wildcard SSL certificates (`*.test`) with green-lock status in Chrome, Firefox, and Edge.

### 3.4 Configuration & Extension Engine
- **FR-4.1**: Template-driven configuration generator for Nginx, Apache, PHP-FPM, and MySQL located in editable `templates/` directory.
- **FR-4.2**: Visual PHP Extension Manager: Toggle common extensions (`xdebug`, `pdo_mysql`, `intl`, `gd`, `zip`, `redis`, `opcache`) via GUI switches, automatically editing `php.ini` and gracefully reloading PHP-FPM.
- **FR-4.3**: Visual PHP Directive Editor: Quick modification of `memory_limit`, `upload_max_filesize`, `max_execution_time`, and `date.timezone`.

### 3.5 Developer Tooling & Productivity
- **FR-5.1**: Embedded database management via Adminer (<500KB single-file PHP) accessible at `http://localhost/adminer` or `http://adminer.test`.
- **FR-5.2**: Desktop Database Client launcher: Auto-detect installed clients (TablePlus, HeidiSQL, DBeaver) and provide 1-click connect with pre-filled credentials (`root` @ `127.0.0.1:3306`).
- **FR-5.3**: Local Mail Catcher: Integrated Mailpit capturing SMTP on port 1025 with web inspection interface on port 8025 (`http://mailpit.test`).
- **FR-5.4**: Isolated Shell Launcher: Open terminal session (PowerShell, CMD, Bash) with pre-configured `PATH` prioritizing active BAMP PHP, MySQL, and Composer binaries.
- **FR-5.5**: Quick App Scaffolder: 1-click provisioning for WordPress, Laravel, and blank PHP apps (creates folder, creates MySQL database, sets up virtual host, issues SSL, and opens in browser).

### 3.6 Diagnostic & Aggregated Logs
- **FR-6.1**: Real-time aggregated log streaming console in the GUI, capturing `stdout` and `stderr` from Nginx, Apache, PHP-FPM, and MySQL.
- **FR-6.2**: Severity filtering (Errors, Warnings, Info) and service-specific filtering tabs.

### 3.7 User Experience & System Tray
- **FR-7.1**: System Tray icon with live status badge and right-click context menu (Start All, Stop All, Quick App, Switch PHP, Open Web Root, Quit).
- **FR-7.2**: Modern high-density developer UI with dark and light theme support.
- **FR-7.3**: Command Palette accessible via `Ctrl+K` / `Cmd+K` for instant keyboard navigation, site search, and service toggling.

## 4. Non-Functional Requirements

- **NFR-1 (Idle Memory Footprint)**: BAMP application GUI and background supervisor must consume under 30MB of RAM when running in system tray.
- **NFR-2 (Startup Time)**: Core application starts in under 500ms on standard SSD hardware.
- **NFR-3 (Clean Teardown)**: Zero orphaned service processes remaining upon exit under all termination scenarios (graceful quit, crash, or SIGTERM).
- **NFR-4 (Zero System Pollution)**: No global system `PATH` mutations; all environment setups remain scoped to BAMP's own lifecycle or child terminal sessions.

## 5. MVP Scope Boundaries

### In Scope for MVP:
- Rust Supervisor managing Nginx, PHP-FPM (default version), and MySQL.
- Embedded DNS resolver for `*.test`.
- Automatic Virtual Host generator from `www/` directory.
- Local Root CA SSL generation and installation.
- Adminer and Mailpit launcher.
- React 19 Dashboard with Start/Stop controls and status indicators.
- System Tray integration.

### Deferred to Post-MVP (Phase 2):
- Concurrent multi-PHP version pooling per site (MVP supports instant global PHP switching; Phase 2 enables concurrent multi-version pools).
- Apache dual-server toggle (MVP defaults to Nginx).
- 1-Click cloud package downloader for new PHP versions (MVP uses drop-in `bin/` folders).
