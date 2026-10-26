# 05 - Configuration Templates & Dynamic Virtual Host Generator

Status: resolved

## Goal
Implement template-driven configuration rendering for Nginx, PHP-FPM, and MySQL, with a file-system watcher that automatically registers and updates virtual hosts when folders in `www/` change.

## Requirements
1. Create default configuration template files:
   - `templates/nginx/nginx.conf.template`
   - `templates/nginx/vhost.conf.template`
   - `templates/php/php-fpm.conf.template`
   - `templates/mysql/my.ini.template`
2. Implement template rendering engine in `src-tauri/src/config/`:
   - Replace placeholders (`{{domain}}`, `{{document_root}}`, `{{fastcgi_pass}}`, `{{ssl_cert}}`, `{{ssl_key}}`).
3. Implement `notify` directory watcher:
   - Watch `www/` directory.
   - Detect project directories, identify public roots (`public/index.php` vs `index.php`).
   - Write rendered server block to `data/nginx/vhosts/<domain>.conf`.
   - Reload Nginx (`nginx -s reload`).
4. Expose Tauri command `get_sites`.

## Completion Criteria
- [x] Creating directory `www/mysite` with `index.php` creates `data/nginx/vhosts/mysite.test.conf`.
- [x] Deleting the directory removes the configuration file.

## Comments
- Authored default template files in `templates/nginx/`, `templates/apache/`, `templates/php/`, and `templates/mysql/`.
- Implemented `ConfigManager` with placeholder substitution (`render_template`) and site auto-discovery with public root detection (`public/index.php` vs `index.php`).
- Implemented `notify` filesystem watcher on `www/` for automatic virtual host re-generation.
- Exposed Tauri commands `get_sites`, `add_linked_folder`, and `render_configs`.
- Verified in automated unit tests (`test_render_template_and_site_detection`).
