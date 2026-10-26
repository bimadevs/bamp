# 02 - Process Supervisor & Anti-Zombie Isolation

Status: resolved

## Goal
Implement the process supervisor in `src-tauri/src/supervisor/` capable of spawning, health-checking, tracking, and cleanly terminating child services without leaving orphaned processes.

## Requirements
1. Implement `Supervisor` struct holding active service instances.
2. Implement cross-platform termination isolation:
   - On Windows: Bind child processes to a Win32 Job Object with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`.
   - On Linux: Spawn child processes with `setpgid` and ensure `SIGTERM` / `SIGKILL` propagates to the entire process group upon exit or application drop.
3. Implement port health checking with exponential backoff probing (`std::net::TcpStream`).
4. Implement Port Conflict Inspector: probe port before spawning and identify conflicting PID/process name if already in use.
5. Implement Tauri IPC commands: `start_service`, `stop_service`, `restart_service`, `start_all_services`, `stop_all_services`, `get_stack_status`.

## Completion Criteria
- [x] Unit tests verify that starting a mock service transitions through `Starting` to `Running`.
- [x] Exiting the parent process terminates mock child processes immediately.

## Comments
- Implemented `ProcessSupervisor` with process lifecycle states (`Stopped`, `Starting`, `Running`, `Stopping`, `Conflict`, `Error`).
- Implemented `platform::configure_process_isolation` and `platform::terminate_process_tree` with `killpg` on Unix and Win32 Job Object on Windows.
- Implemented `Drop` on `ProcessSupervisor` guaranteeing zero-zombie process termination.
- Implemented `inspector::inspect_port` probing socket occupation and reading `/proc/net/tcp` socket inodes & PID/process name on Linux (`netstat` on Windows).
- Implemented and wired Tauri IPC commands (`start_service`, `stop_service`, `restart_service`, `start_all_services`, `stop_all_services`, `get_stack_status`) in `src-tauri/src/lib.rs`.
- All unit tests verified and passed via `cargo test`.
