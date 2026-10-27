# BAMP Binary Provisioning Scripts

This folder contains scripts to manage portable service binaries for BAMP without bloating the Git repository.

## 1. Full Stack Downloader (`install-binaries-win.ps1`)

Downloads and installs the complete latest stable & LTS stack into `bin/`:
- **Nginx** (1.26.x) $\rightarrow$ `bin/nginx/`
- **Apache HTTPD** (2.4.x) $\rightarrow$ `bin/apache/`
- **PHP 8.3 NTS** (VS16 x64) $\rightarrow$ `bin/php/php-8.3/`
- **MariaDB 10.11 LTS** (Win64) $\rightarrow$ `bin/mysql/`
- **Mailpit** (Windows amd64) $\rightarrow$ `bin/mailpit/`

```powershell
powershell -ExecutionPolicy Bypass -File scripts/install-binaries-win.ps1
```

## 2. Specific Service & Version Switcher (`install-service-version.ps1`)

Used to install a specific version of PHP or another service on-demand (e.g. via BAMP's Package Hub):

```powershell
# Install PHP 8.4
powershell -ExecutionPolicy Bypass -File scripts/install-service-version.ps1 -Php 8.4

# Install PHP 8.2
powershell -ExecutionPolicy Bypass -File scripts/install-service-version.ps1 -Php 8.2

# Install PHP 8.1
powershell -ExecutionPolicy Bypass -File scripts/install-service-version.ps1 -Php 8.1

# Install PHP 7.4 (Legacy)
powershell -ExecutionPolicy Bypass -File scripts/install-service-version.ps1 -Php 7.4
```

## Binary Layout in `bin/`

- `bin/nginx/nginx.exe`
- `bin/apache/bin/httpd.exe` (or `bin/apache/httpd.exe`)
- `bin/php/php-8.3/php-cgi.exe` (and other version folders like `php-8.4/`, `php-8.2/`, `php-7.4/`)
- `bin/mysql/bin/mariadbd.exe` (or `mysqld.exe`)
- `bin/mailpit/mailpit.exe`
