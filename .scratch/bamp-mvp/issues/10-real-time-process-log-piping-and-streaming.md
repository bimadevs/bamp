# 10 - Piped Stdout/Stderr & Real-time Log Streaming

Status: resolved

## Goal
Pipe stdout and stderr from all spawned child processes into Tauri streaming events and log files in `data/logs/<service>.log`, displaying live server outputs in the frontend Activity Log drawer.

## Requirements
1. Supervisor process piping:
   - Configure `Stdio::piped()` for child process stdout and stderr.
   - Spawn asynchronous reader tasks to read line-by-line without blocking.
   - Emit Tauri event `log-stream` containing `{ service, level, msg }`.
   - Append lines to `data/logs/<service>.log`.
2. Frontend integration:
   - Listen for `log-stream` events in `App.tsx`.
   - Auto-scroll and color-code live log stream with service filters.

## Completion Criteria
- [x] Child process stdout and stderr are captured and written to disk.
- [x] Tauri event `log-stream` fires and displays in React Activity Logs console.
- [x] Filter tabs dynamically filter messages per service.

## Comments
- Implemented background thread reader tasks consuming piped stdout/stderr.
- Integrated `app.emit("log-stream", ...)` via `tauri::Emitter`.
- Added auto-scroll and service filtering in `App.tsx`.
