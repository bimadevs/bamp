# 09 - Multi-Port Dynamic PHP Pools & Persistent Settings

Status: resolved

## Goal
Implement persistent settings storage in `data/settings.json` and multi-port dynamic PHP pools (binding specific FastCGI ports per PHP version in virtual host configs).

## Requirements
1. Settings persistence:
   - Store settings in `data/settings.json` (active_web_server, default_php, linked_folders, site_php_overrides).
   - Load settings on BAMP startup; save changes atomically.
2. Dynamic FastCGI port mapping:
   - Map PHP versions to dedicated ports: default -> 9000, php-8.4 -> 9084, php-8.3 -> 9083, php-8.2 -> 9082, php-7.4 -> 9074.
   - In `render_all_configs`, substitute `{{fastcgi_pass}}` dynamically per site based on its configured PHP version.
3. Multi-pool supervisor integration:
   - Allow starting separate PHP-FPM pools on demand when sites require different versions.

## Completion Criteria
- [x] Settings persist cleanly to `data/settings.json` and reload upon application launch.
- [x] Sites configured with custom PHP versions render distinct FastCGI ports in `data/nginx/vhosts/*.conf`.
- [x] Unit tests for `AppSettings` and `php_version_to_port` pass.

## Comments
- Implemented `AppSettings` struct with JSON serialization and deserialization.
- Implemented `php_version_to_port` mapping versions (php-8.4, php-8.3, php-8.2, php-8.1, php-7.4) to ports 9084, 9083, 9082, 9081, 9074.
- Updated `render_all_configs` to substitute `fastcgi_pass` per virtual host.
