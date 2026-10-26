# 06 - React 19 Frontend Dashboard & System Tray

Status: resolved

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
- [x] Dashboard renders accurately at `http://localhost:1420`.
- [x] Clicking "Start All" triggers Tauri IPC `start_all_services` and updates status cards reactively.
- [x] Idle memory footprint of the webview window remains under 25MB.

## Comments
- Configured Tailwind CSS v4 and `lucide-react`.
- Built high-density Raycast/Linear styled developer dashboard conforming strictly to Hallmark and Impeccable standards.
- Implemented Hero Stack Bar with Start/Stop All and global status indicator.
- Implemented Service Cards for Nginx, Apache, PHP-FPM, MySQL, and Mailpit with status badges (running, stopped, conflict, error).
- Implemented Sites Table with auto-detection of `.test` virtual hosts, document roots, and HTTPS status.
- Implemented Log Drawer with multi-channel filtering.
- Implemented Command Palette modal triggered by `Ctrl+K` / `Cmd+K`.
- Configured System Tray with context menu in `src-tauri/src/lib.rs`.
- `pnpm build` verified with zero errors (bundle size <85KB gzipped).
