# 08 - Active Web Server Switcher & Graceful VHost Reload

Status: resolved

## Goal
Implement exclusive Web Server switching between Nginx (default) and Apache on port 80, and ensure automatic graceful reload (`nginx -s reload` / `httpd -k graceful`) when virtual hosts are added or modified.

## Requirements
1. Active Web Server state:
   - Provide setting `active_web_server` ("nginx" | "apache") in the backend.
   - When `start_all_services` runs, spawn only the active web server, preventing port 80 conflict.
   - Expose Tauri IPC commands: `get_active_web_server` and `set_active_web_server(server_id)`.
   - When switching web servers, cleanly stop the old server and start the new server on port 80.
2. Graceful reload on VHost modification:
   - In `ConfigManager::start_watcher`, when new virtual hosts are rendered, trigger graceful reload on the active web server.
3. Frontend UI:
   - In `App.tsx`, provide segmented toggle control: [Nginx (Active) | Apache].
   - Reflect real-time running status on the active web server card.

## Completion Criteria
- [x] `set_active_web_server("apache")` stops Nginx and starts Apache on port 80.
- [x] `set_active_web_server("nginx")` stops Apache and starts Nginx on port 80.
- [x] File creation in `www/` triggers automatic virtual host generation and server reload.
- [x] `cargo check` and `pnpm build` pass with zero errors.

## Comments
- Implemented `active_web_server` in `ProcessSupervisor` and segmented control in `App.tsx`.
- Updated `start_all_services` to only boot the active web server and place the other in standby.
- Integrated `reload_web_server` invoked directly from `start_watcher` in `ConfigManager`.
