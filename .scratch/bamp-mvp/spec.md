# Spec: BAMP MVP (Minimum Viable Product)

## Overview

The BAMP MVP delivers a fully functional local development manager capable of running an Nginx + PHP-FPM + MySQL stack with an embedded DNS resolver for `*.test` domains, automated virtual hosts, local SSL certificates, and a lightweight React 19 dashboard.

## Key Deliverables

1. **Rust Supervisor Core**:
   - Spawns and manages Nginx, PHP-FPM, MySQL, and Mailpit.
   - Enforces zero-zombie process termination via Job Objects (Windows) or Process Groups (Linux).
   - Exposes typed Tauri IPC commands for starting, stopping, and inspecting stack status.

2. **Embedded DNS Resolver**:
   - UDP socket on `127.0.0.1:53` returning `127.0.0.1` for any `*.test` domain query.
   - Fallback mode logging port conflicts gracefully.

3. **Local Root CA & SSL**:
   - Generates local Root CA and issues wildcard `*.test` SSL certificates.
   - Saves certificates to `data/ssl/` for Nginx consumption.

4. **Template-Driven Virtual Host Generator**:
   - Scans `www/` directory.
   - Renders Nginx server blocks mapping `<folder>.test` to `<folder>` (or `<folder>/public`).
   - Reloads Nginx on folder changes.

5. **React 19 Frontend Dashboard**:
   - Hero "Start All" / "Stop All" controls.
   - Real-time status cards for each service with port and PID.
   - Sites table listing active `.test` domains with 1-click "Open in Browser".
   - Aggregated real-time log drawer.
   - System tray menu with quick start/stop actions.

6. **Tooling Integration**:
   - Single-file Adminer database manager served at `localhost/adminer` or `adminer.test`.
   - Mailpit integration on port 1025 (SMTP) and 8025 (Web UI).

## Acceptance Criteria

- [x] `bamp` launches in under 500ms and consumes < 30MB RAM in idle state.
- [x] Clicking "Start All" successfully transitions all available services to `Running`.
- [x] Placing a test file `www/demo/index.php` with `<?php echo "Hello BAMP"; ?>` makes `http://demo.test` and `https://demo.test` immediately accessible in the browser without manual server restart.
- [x] Closing BAMP terminates all Nginx, PHP, and MySQL processes immediately with zero orphaned processes.
