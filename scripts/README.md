# BAMP Binary Provisioning Scripts

This folder contains scripts to download and configure portable service binaries for BAMP without bloating the Git repository.

## Windows

To download all required Windows binaries (Nginx, Apache, PHP 8.3 NTS, MariaDB, Mailpit) into `bin/`:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/install-binaries-win.ps1
```

Or selectively:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/install-binaries-win.ps1 -Services nginx,php,mysql,mailpit
```

## Binary Layout in `bin/`

When installed, the binaries are structured as follows:

- `bin/nginx/nginx.exe`
- `bin/apache/bin/httpd.exe` (or `bin/apache/httpd.exe`)
- `bin/php/php-8.3/php-cgi.exe` (and `php.exe`, `ext/`, `php.ini`)
- `bin/mysql/bin/mariadbd.exe` (or `mysqld.exe`)
- `bin/mailpit/mailpit.exe`
