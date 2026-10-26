# 12 - Isolated Shell & Smart Database Client Launcher

Status: resolved

## Goal
Implement one-click Isolated Terminal launching with PATH injected to BAMP binaries, and smart desktop database client auto-detection with fallback to Adminer.

## Requirements
1. Isolated Shell:
   - Command `launch_isolated_terminal`.
   - On Windows: launch `wt.exe`, `powershell.exe`, or `cmd.exe` with prepended PATH.
   - On Linux: launch detected desktop terminal (`x-terminal-emulator`, `gnome-terminal`, `konsole`, `xterm`, etc.) with prepended PATH.
2. Smart DB Launcher:
   - Command `launch_database_client`.
   - Probe for TablePlus, DBeaver, HeidiSQL, Beekeeper Studio.
   - If found, open connection to `127.0.0.1:3306` user `root`.
   - If not found, open `http://adminer.test` in browser.
3. UI integrations in toolbar and Command Palette.

## Completion Criteria
- [x] Isolated Terminal command launches native OS terminal with injected PATH.
- [x] Database button connects to desktop client or opens Adminer.
- [x] Commands integrated into Command Palette (`Ctrl+K`).

## Comments
- Implemented `ToolsManager::launch_isolated_terminal` supporting Windows Terminal, PowerShell, CMD, and Linux desktop terminal emulators.
- Implemented `ToolsManager::launch_database_client` with auto-probe and Adminer fallback.
- Added toolbar buttons and shortcuts in `App.tsx`.
