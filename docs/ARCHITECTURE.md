# BAMP: System Architecture

## 1. Architectural Overview

BAMP is structured as a two-tier desktop application:
1. **Core Layer (Rust / Tauri v2)**: Responsible for system-level operations: child process management, socket binding, DNS resolution, SSL root certificate generation, file system watching, and platform-specific API calls.
2. **Presentation Layer (React 19 / TypeScript / Vite)**: Responsible for the user dashboard, interactive service controls, configuration editing, log visualization, and notification rendering.

Communication between layers occurs strictly via Tauri IPC (Commands for request-response operations and Events for asynchronous telemetry like log streams and status changes).

```mermaid
graph TD
    UI["React 19 Frontend<br/>(Dashboard, Sites, Logs, Tray Menu)"]
    IPC["Tauri IPC Bridge<br/>(Typed Commands & Real-time Events)"]
    
    subgraph RustCore ["BAMP Rust Core (src-tauri)"]
        Supervisor["Process Supervisor<br/>(Job Objects & Process Groups)"]
        DNS["Embedded DNS Server<br/>(UDP 127.0.0.1:53)"]
        SSL["SSL Root CA Engine<br/>(rcgen + Trust Store Installer)"]
        ConfigGen["Config & VHost Generator<br/>(Template Engine)"]
        Inspector["Port Conflict Inspector<br/>(Socket Probe & PID Lookup)"]
        Watcher["File System Watcher<br/>(notify crate)"]
    end
    
    subgraph PortableBinaries ["Service Binaries (bin/)"]
        Nginx["Nginx Web Server"]
        PHPFPM["PHP-FPM Pools"]
        MySQL["MySQL / MariaDB"]
        Mailpit["Mailpit (SMTP & Web)"]
    end
    
    UI <--> IPC
    IPC <--> RustCore
    Supervisor --> PortableBinaries
    ConfigGen -. Generates .-> PortableBinaries
    Watcher --> ConfigGen
    DNS -. Resolves *.test .-> Nginx
```

## 2. Subsystems & Responsibilities

### 2.1 Process Supervisor (`src-tauri/src/supervisor/`)
- Spawns and manages service binaries located in `bin/`.
- **Windows**: Attaches all child processes to a single Win32 Job Object with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`. When the BAMP main process exits, the Windows kernel forcefully and cleanly terminates all descendant processes.
- **Linux**: Spawns processes in dedicated process groups (`setpgid`) and catches termination signals (SIGINT, SIGTERM) to dispatch `SIGTERM` to the process group, escalating to `SIGKILL` after a grace period.
- Exposes structured states: `Stopped`, `Starting`, `Running(PID, Port)`, `Error(String)`.

### 2.2 Embedded DNS Server (`src-tauri/src/dns/`)
- Listens on `127.0.0.1:53` (UDP).
- Inspects incoming DNS queries:
  - If query domain ends with `.test` (e.g. `laravel-app.test`, `my-site.test`), immediately returns an `A` record pointing to `127.0.0.1` with a low TTL (60s).
  - All other queries are ignored or forwarded to system upstream DNS.
- Fallback mode: If port 53 is already bound by another service (e.g., systemd-resolved), BAMP logs a warning and falls back to syncing active domains into `/etc/hosts` or `C:\Windows\System32\drivers\etc\hosts`.

### 2.3 SSL Engine (`src-tauri/src/ssl/`)
- Generates a local root Certificate Authority (BAMP Root CA) using RSA 2048 or ECDSA P-256.
- Installs the root certificate into:
  - Windows: Windows Certificate Store (`Root` store via `certutil` or Win32 CryptoAPI).
  - Linux: System trust directory (`/usr/local/share/ca-certificates/` or `/etc/ca-certificates/trust-source/anchors/`) and executes update tools (`update-ca-certificates`).
- Issues a wildcard certificate for `*.test` signed by the BAMP Root CA and places it in `data/ssl/cert.pem` and `data/ssl/key.pem` for Nginx and Apache to consume.

### 2.4 Configuration & Virtual Host Generator (`src-tauri/src/config/`)
- Reads templates from `templates/` directory.
- Watches `www/` and registered Linked Folders using the `notify` file-system watcher.
- When a folder is added, deleted, or renamed:
  1. Derives the site domain name (folder name sanitized, appending `.test`).
  2. Detects public entry point (`public/index.php`, `index.php`, or `index.html`).
  3. Renders a new virtual host configuration snippet into `data/nginx/vhosts/<site>.conf`.
  4. Triggers a graceful reload (`nginx -s reload`).

### 2.5 Port Conflict Inspector (`src-tauri/src/inspector/`)
- Before executing a service start sequence, probes the target ports (`80`, `443`, `3306`, `8025`, `1025`).
- If a port is occupied:
  - Queries OS socket tables to retrieve the owning Process ID (PID) and process executable name (e.g., `PID 4120 (inetinfo.exe / IIS)` or `PID 892 (mysqld)`).
  - Emits a diagnostic warning to the frontend allowing the user to inspect or terminate the conflicting process.

## 3. Directory Layout Conventions

```
bamp-root/
├── bamp (or bamp.exe)        # Tauri Native Application Core
├── bin/                      # Portable Executable Binaries
│   ├── nginx/                # Nginx binary and static modules
│   ├── apache/               # Apache httpd binary
│   ├── php/
│   │   ├── php-8.3/          # PHP 8.3 CLI, PHP-FPM, php.ini
│   │   └── php-7.4/          # PHP 7.4 runtime
│   ├── mysql/                # MySQL / MariaDB server binaries
│   └── mailpit/              # Mailpit executable
├── data/                     # Dynamic Runtime Data
│   ├── mysql/                # MySQL database files & schemas
│   ├── ssl/                  # BAMP Root CA and *.test certificates
│   └── logs/                 # Service output logs (nginx, php, mysql)
├── templates/                # User-Customizable Configuration Templates
│   ├── nginx/
│   │   ├── nginx.conf.template
│   │   └── vhost.conf.template
│   ├── php/
│   │   └── php-fpm.conf.template
│   └── mysql/
│       └── my.ini.template
└── www/                      # Default Local Web Root
    ├── default/              # Welcome page and info
    └── sample-project/       # User project directories
```

## 4. Platform Abstraction Layer (PAL)

All platform-divergent code is isolated in Rust modules under `src-tauri/src/platform/`:
- `platform::process`: Windows Job Objects vs Linux `libc::setpgid` and signal forwarding.
- `platform::ssl`: Windows `certutil` / CryptoAPI vs Linux `update-ca-certificates`.
- `platform::hosts`: Windows permissions and line endings vs Unix permissions.
- `platform::shell`: Spawning `powershell.exe` / `cmd.exe` vs `/bin/bash` with environment variables.
