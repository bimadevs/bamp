# BAMP: Project Context & Philosophy

## Context

Web developers working with PHP, WordPress, Laravel, and MySQL have historically relied on local development environments like XAMPP, WampServer, and Laragon. While these tools have served developers for years, they suffer from fundamental architectural limitations:

1. **Monolithic Bloat**: XAMPP bundles all services into a heavy, inflexible ~150MB+ installer that hardcodes binaries to fixed paths, making PHP version switching painful or impossible without manual file surgery.
2. **Zombie Processes & Port Clashes**: Legacy managers spawn raw child processes that frequently become orphaned when the manager crashes or closes, leaving ports 80 and 3306 locked and confusing users.
3. **UAC Friction & Hosts Mutation**: Adding a virtual host traditionally requires mutating system `hosts` files, triggering repeated administrative elevation (UAC/sudo) prompts and anti-virus warnings.
4. **Untrusted Local SSL**: Standard self-signed certificates generate browser security warnings that break modern browser APIs (HTTP/2, Service Workers, Secure Cookies).
5. **Abandonment or Platform Lock-in**: Laragon is Windows-only, closed-source, and has seen slow maintenance; Laravel Herd is proprietary and macOS/Windows-centric with commercial paywalls; LocalWP is built on Electron consuming 300MB+ of idle RAM.

## The BAMP Solution

**BAMP** (Bima Apache/Nginx MySQL PHP Manager) is an open-source, ultra-lightweight, and modular local web development stack manager.

### Core Philosophy

1. **Lightweight First (<30MB Idle RAM)**: Built on Tauri v2 (Rust backend + React 19 frontend), avoiding Electron's massive memory overhead.
2. **Zero-Core Portability**: The core application binary is tiny (~15MB). Service binaries (PHP, MySQL, Nginx, Apache, Mailpit) live in a modular `bin/` directory that can be dropped in or downloaded on demand.
3. **Zero Zombie Guarantee**: The Rust Supervisor ties all child processes to Win32 Job Objects (on Windows) or Process Groups (on Linux), guaranteeing that when BAMP terminates, every managed process terminates instantly.
4. **Zero-Touch Local Domains**: An embedded UDP DNS resolver intercepts `*.test` queries locally, providing instant zero-configuration domains for every project folder without touching system `hosts` files.
5. **Green-Lock Local SSL**: Built-in Local Root CA engine installs once and issues trusted HTTPS certificates on the fly.
6. **Concurrent PHP-FPM Pools**: Run legacy projects (PHP 7.4) and modern projects (PHP 8.3) simultaneously using dedicated fastcgi sockets.
