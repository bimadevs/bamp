# 06 - React 19 Frontend Dashboard & System Tray

Status: ready-for-agent

## Goal
Build the modern, ultra-lightweight developer interface in React 19, TypeScript, and Tailwind CSS, integrating with Tauri's IPC commands, real-time events, and System Tray.

## Requirements
1. Install and configure Tailwind CSS and `lucide-react` in the frontend.
2. Build UI views following modern developer tool aesthetics (Raycast/Linear style):
   - **Hero Stack Bar**: Global "Start All" / "Stop All" button with aggregate stack status indicator.
   - **Service Status Cards**: Individual cards for Nginx, PHP-FPM, MySQL, Mailpit, and DNS displaying status badge, PID, port, and start/stop/restart buttons.
   - **Sites Table**: List of active virtual hosts with domain name, local folder path, SSL status, and 1-click "Open in Browser" / "Open in VS Code" actions.
   - **Log Drawer**: Aggregated live log console with filter tabs (All, Nginx, PHP, MySQL) and auto-scroll.
   - **Command Palette (`Ctrl+K` / `Cmd+K`)**: Quick fuzzy launcher to search sites and toggle services.
3. Configure Tauri System Tray:
   - Tray icon with status color.
   - Context menu: Start All, Stop All, Open Dashboard, Open www folder, Quit.

## Completion Criteria
- Dashboard renders accurately at `http://localhost:1420`.
- Clicking "Start All" triggers Tauri IPC `start_all_services` and updates status cards reactively.
- Idle memory footprint of the webview window remains under 25MB.

## Comments
