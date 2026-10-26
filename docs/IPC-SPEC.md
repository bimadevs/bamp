# BAMP: Tauri IPC Specification

This document defines the interface contract between the Rust backend (`src-tauri`) and the React 19 frontend (`src/`).

## 1. Tauri Commands (Frontend -> Backend)

### 1.1 Service Controls

#### `start_all_services`
Starts all configured services in dependency order (DNS -> MySQL -> PHP-FPM -> Nginx/Apache -> Mailpit).
- **Parameters**: None
- **Returns**: `Result<StackStatus, String>`

#### `stop_all_services`
Stops all running services gracefully.
- **Parameters**: None
- **Returns**: `Result<StackStatus, String>`

#### `start_service`
Starts a specific service.
- **Parameters**:
  - `service_id`: `String` (`"nginx"` | `"apache"` | `"php"` | `"mysql"` | `"mailpit"` | `"dns"`)
- **Returns**: `Result<ServiceState, String>`

#### `stop_service`
Stops a specific service.
- **Parameters**:
  - `service_id`: `String`
- **Returns**: `Result<ServiceState, String>`

#### `restart_service`
Restarts a specific service.
- **Parameters**:
  - `service_id`: `String`
- **Returns**: `Result<ServiceState, String>`

#### `get_stack_status`
Retrieves the real-time status of all services and active ports.
- **Parameters**: None
- **Returns**: `Result<StackStatus, String>`

```typescript
export interface ServiceState {
  id: "nginx" | "apache" | "php" | "mysql" | "mailpit" | "dns";
  name: string;
  status: "stopped" | "starting" | "running" | "stopping" | "conflict" | "error";
  pid?: number;
  port?: number;
  version?: string;
  error?: string;
}

export interface StackStatus {
  isRunning: boolean;
  services: Record<string, ServiceState>;
}
```

---

### 1.2 Virtual Hosts & Sites

#### `get_sites`
Lists all detected and configured virtual hosts.
- **Parameters**: None
- **Returns**: `Result<Site[], String>`

#### `add_linked_folder`
Registers an external project directory for auto-vhost monitoring.
- **Parameters**:
  - `path`: `String`
- **Returns**: `Result<Site[], String>`

#### `set_site_php_version`
Overrides the PHP version for a specific site.
- **Parameters**:
  - `site_id`: `String`
  - `php_version`: `String` (e.g. `"php-8.3"` or `"default"`)
- **Returns**: `Result<Site, String>`

```typescript
export interface Site {
  id: string;
  name: string;
  domain: string;
  path: string;
  documentRoot: string;
  phpVersion: string;
  hasSsl: boolean;
  status: "active" | "error";
}
```

---

### 1.3 PHP Extensions & Directives

#### `get_php_extensions`
Retrieves list of available and active PHP extensions.
- **Parameters**:
  - `php_version`: `String` (optional, defaults to active version)
- **Returns**: `Result<PhpExtension[], String>`

#### `toggle_php_extension`
Enables or disables a PHP extension in `php.ini`.
- **Parameters**:
  - `extension_name`: `String` (e.g. `"xdebug"`, `"pdo_mysql"`)
  - `enabled`: `boolean`
  - `php_version`: `String`
- **Returns**: `Result<boolean, String>`

---

### 1.4 Utilities & Tooling

#### `open_in_browser`
Opens a URL in the user's default web browser.
- **Parameters**:
  - `url`: `String`
- **Returns**: `Result<(), String>`

#### `open_isolated_terminal`
Spawns a terminal session with BAMP's `bin/` added to the `PATH`.
- **Parameters**:
  - `cwd`: `String` (optional target directory)
- **Returns**: `Result<(), String>`

#### `scaffold_quick_app`
Executes automated project generation (WordPress / Laravel / Blank).
- **Parameters**:
  ```typescript
  export interface QuickAppPayload {
    appType: "wordpress" | "laravel" | "blank";
    name: string;
    targetPath?: string;
    createDatabase: boolean;
    dbName?: string;
    phpVersion?: string;
  }
  ```
- **Returns**: `Result<Site, String>`

---

## 2. Tauri Events (Backend -> Frontend Streaming)

### `bamp://service-status-changed`
Emitted whenever a service changes state.
- **Payload**: `ServiceState`

### `bamp://log-entry`
Real-time log message streamed from service processes.
- **Payload**:
  ```typescript
  export interface LogEntry {
    serviceId: string;
    timestamp: string;
    level: "info" | "warn" | "error";
    message: string;
  }
  ```

### `bamp://port-conflict`
Emitted when a service fails to start due to an occupied port.
- **Payload**:
  ```typescript
  export interface PortConflictPayload {
    serviceId: string;
    port: number;
    conflictingPid?: number;
    conflictingProcessName?: string;
  }
  ```
