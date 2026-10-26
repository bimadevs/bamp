//! Process Supervisor for BAMP services (Nginx, Apache, PHP-FPM, MySQL, Mailpit).
//! Handles process lifecycle, health-checking, port conflict inspection, and anti-zombie lifetime management.

use crate::inspector::inspect_port;
use crate::platform::{attach_child_isolation, configure_process_isolation, resolve_binary, terminate_process_tree};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::TcpStream;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::Mutex;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ServiceStatus {
    Stopped,
    Starting,
    Running,
    Stopping,
    Conflict,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceState {
    pub id: String,
    pub name: String,
    pub status: ServiceStatus,
    pub pid: Option<u32>,
    pub port: Option<u16>,
    pub version: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackStatus {
    pub is_running: bool,
    pub services: HashMap<String, ServiceState>,
}

#[derive(Clone)]
pub struct ServiceDefinition {
    pub id: String,
    pub name: String,
    pub binary_candidates: Vec<String>,
    pub default_port: Option<u16>,
    pub args: Vec<String>,
    pub work_dir: Option<PathBuf>,
    pub env: HashMap<String, String>,
}

struct ActiveChild {
    child: Child,
    pid: u32,
    #[allow(dead_code)]
    port: Option<u16>,
}

pub struct ProcessSupervisor {
    base_dir: PathBuf,
    active_web_server: Mutex<String>,
    app_handle: Mutex<Option<tauri::AppHandle>>,
    definitions: Mutex<HashMap<String, ServiceDefinition>>,
    children: Mutex<HashMap<String, ActiveChild>>,
    states: Mutex<HashMap<String, ServiceState>>,
}

impl ProcessSupervisor {
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            base_dir,
            active_web_server: Mutex::new("nginx".to_string()),
            app_handle: Mutex::new(None),
            definitions: Mutex::new(HashMap::new()),
            children: Mutex::new(HashMap::new()),
            states: Mutex::new(HashMap::new()),
        }
    }

    pub fn set_app_handle(&self, handle: tauri::AppHandle) {
        *self.app_handle.lock().unwrap() = Some(handle);
    }

    /// Get current active web server id ("nginx" or "apache")
    pub fn get_active_web_server(&self) -> String {
        self.active_web_server.lock().unwrap().clone()
    }

    /// Set active web server and smoothly switch if currently running
    pub fn set_active_web_server(&self, server_id: &str) -> Result<String, String> {
        if server_id != "nginx" && server_id != "apache" {
            return Err("Supported web servers are 'nginx' and 'apache'".to_string());
        }

        let old_server = self.get_active_web_server();
        if old_server == server_id {
            return Ok(server_id.to_string());
        }

        // Check if old server was running
        let was_running = {
            let states = self.states.lock().unwrap();
            states.get(&old_server).map(|s| s.status == ServiceStatus::Running).unwrap_or(false)
        };

        if was_running {
            let _ = self.stop_service(&old_server);
        }

        *self.active_web_server.lock().unwrap() = server_id.to_string();

        if was_running {
            let _ = self.start_service(server_id);
        }

        Ok(server_id.to_string())
    }

    /// Graceful reload of the active web server process
    pub fn reload_web_server(&self) -> Result<(), String> {
        let active = self.get_active_web_server();
        let is_running = {
            let states = self.states.lock().unwrap();
            states.get(&active).map(|s| s.status == ServiceStatus::Running).unwrap_or(false)
        };

        if !is_running {
            return Ok(());
        }

        if active == "nginx" {
            if let Some(bin) = resolve_binary(&self.base_dir, "nginx", &["nginx"]) {
                let _ = Command::new(bin)
                    .args(["-p", ".", "-c", "data/nginx/nginx.conf", "-s", "reload"])
                    .current_dir(&self.base_dir)
                    .status();
            }
        } else if active == "apache" {
            if let Some(bin) = resolve_binary(&self.base_dir, "apache", &["httpd", "apache2"]) {
                let _ = Command::new(bin)
                    .args(["-f", "data/apache/httpd.conf", "-k", "graceful"])
                    .current_dir(&self.base_dir)
                    .status();
            }
        }
        Ok(())
    }

    /// Register a service definition.
    pub fn register_service(&self, def: ServiceDefinition) {
        let initial_state = ServiceState {
            id: def.id.clone(),
            name: def.name.clone(),
            status: ServiceStatus::Stopped,
            pid: None,
            port: def.default_port,
            version: None,
            error: None,
        };

        self.definitions.lock().unwrap().insert(def.id.clone(), def);
        self.states.lock().unwrap().insert(initial_state.id.clone(), initial_state);
    }

    /// Get current status of all services.
    pub fn get_stack_status(&self) -> StackStatus {
        let mut states = self.states.lock().unwrap().clone();
        let mut children = self.children.lock().unwrap();

        // Check if any child has unexpectedly exited
        for (id, active) in children.iter_mut() {
            match active.child.try_wait() {
                Ok(Some(status)) => {
                    if let Some(state) = states.get_mut(id) {
                        state.status = ServiceStatus::Stopped;
                        state.pid = None;
                        if !status.success() {
                            state.error = Some(format!("Process exited with status {:?}", status.code()));
                        }
                    }
                }
                Ok(None) => {} // still running
                Err(e) => {
                    if let Some(state) = states.get_mut(id) {
                        state.status = ServiceStatus::Error;
                        state.error = Some(format!("Error polling process: {}", e));
                    }
                }
            }
        }

        // Clean up exited children
        children.retain(|_, active| match active.child.try_wait() {
            Ok(Some(_)) => false,
            _ => true,
        });

        // Update stored states with clean values
        *self.states.lock().unwrap() = states.clone();

        let is_running = states.values().any(|s| s.status == ServiceStatus::Running);
        StackStatus { is_running, services: states }
    }

    /// Start a service by ID.
    pub fn start_service(&self, id: &str) -> Result<ServiceState, String> {
        let def = self
            .definitions
            .lock()
            .unwrap()
            .get(id)
            .cloned()
            .ok_or_else(|| format!("Unknown service: {}", id))?;

        // 1. Check if already running
        {
            let mut children = self.children.lock().unwrap();
            if let Some(active) = children.get_mut(id) {
                if let Ok(None) = active.child.try_wait() {
                    return Ok(self.states.lock().unwrap().get(id).cloned().unwrap());
                }
            }
        }

        // 2. Check for port conflicts
        if let Some(port) = def.default_port {
            let conflict = inspect_port(port);
            if conflict.is_occupied {
                let err_msg = match (&conflict.process_name, conflict.pid) {
                    (Some(name), Some(pid)) => format!("Port {} is already in use by {} (PID {})", port, name, pid),
                    _ => format!("Port {} is already in use by another process", port),
                };
                let mut state = self.states.lock().unwrap().get(id).cloned().unwrap();
                state.status = ServiceStatus::Conflict;
                state.error = Some(err_msg.clone());
                self.states.lock().unwrap().insert(id.to_string(), state.clone());
                return Err(err_msg);
            }
        }

        // 3. Resolve binary
        let binary_refs: Vec<&str> = def.binary_candidates.iter().map(|s| s.as_str()).collect();
        let binary_path = resolve_binary(&self.base_dir, &def.id, &binary_refs).ok_or_else(|| {
            let err = format!("Binary for service '{}' not found in bin/ or PATH", def.name);
            let mut state = self.states.lock().unwrap().get(id).cloned().unwrap();
            state.status = ServiceStatus::Error;
            state.error = Some(err.clone());
            self.states.lock().unwrap().insert(id.to_string(), state);
            err
        })?;

        // 4. Mark Starting
        {
            let mut states = self.states.lock().unwrap();
            if let Some(state) = states.get_mut(id) {
                state.status = ServiceStatus::Starting;
                state.error = None;
            }
        }

        // 5. Spawn process with isolation
        let mut cmd = Command::new(&binary_path);
        cmd.args(&def.args);
        if let Some(ref work_dir) = def.work_dir {
            cmd.current_dir(work_dir);
        }
        for (k, v) in &def.env {
            cmd.env(k, v);
        }

        configure_process_isolation(&mut cmd);

        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| {
            let err = format!("Failed to spawn {}: {}", def.name, e);
            let mut states = self.states.lock().unwrap();
            if let Some(state) = states.get_mut(id) {
                state.status = ServiceStatus::Error;
                state.error = Some(err.clone());
            }
            err
        })?;

        let pid = child.id();
        let _ = attach_child_isolation(pid);

        // Pipe stdout and stderr asynchronously
        if let Some(stdout) = child.stdout.take() {
            let id_clone = id.to_string();
            let app_opt = self.app_handle.lock().unwrap().clone();
            let base_clone = self.base_dir.clone();
            std::thread::spawn(move || {
                use std::io::{BufRead, BufReader, Write};
                let reader = BufReader::new(stdout);
                let log_file_path = base_clone.join("data/logs").join(format!("{}.log", id_clone));
                let _ = std::fs::create_dir_all(base_clone.join("data/logs"));
                let mut file = std::fs::OpenOptions::new().create(true).append(true).open(&log_file_path).ok();
                for line in reader.lines().flatten() {
                    if let Some(ref mut f) = file {
                        let _ = writeln!(f, "[stdout] {}", line);
                    }
                    if let Some(ref app) = app_opt {
                        use tauri::Emitter;
                        let _ = app.emit("log-stream", serde_json::json!({
                            "service": id_clone,
                            "level": "info",
                            "msg": line,
                        }));
                    }
                }
            });
        }

        if let Some(stderr) = child.stderr.take() {
            let id_clone = id.to_string();
            let app_opt = self.app_handle.lock().unwrap().clone();
            let base_clone = self.base_dir.clone();
            std::thread::spawn(move || {
                use std::io::{BufRead, BufReader, Write};
                let reader = BufReader::new(stderr);
                let log_file_path = base_clone.join("data/logs").join(format!("{}.log", id_clone));
                let _ = std::fs::create_dir_all(base_clone.join("data/logs"));
                let mut file = std::fs::OpenOptions::new().create(true).append(true).open(&log_file_path).ok();
                for line in reader.lines().flatten() {
                    if let Some(ref mut f) = file {
                        let _ = writeln!(f, "[stderr] {}", line);
                    }
                    if let Some(ref app) = app_opt {
                        use tauri::Emitter;
                        let _ = app.emit("log-stream", serde_json::json!({
                            "service": id_clone,
                            "level": "error",
                            "msg": line,
                        }));
                    }
                }
            });
        }

        // 6. If service has a port, probe health
        if let Some(port) = def.default_port {
            let healthy = Self::probe_port_health(port, 30, 100);
            if !healthy {
                // Kill process if it failed to bind port
                let _ = terminate_process_tree(pid, true);
                let err = format!("Service {} started (PID {}) but failed to open port {}", def.name, pid, port);
                let mut states = self.states.lock().unwrap();
                if let Some(state) = states.get_mut(id) {
                    state.status = ServiceStatus::Error;
                    state.error = Some(err.clone());
                }
                return Err(err);
            }
        }

        let running_state = ServiceState {
            id: id.to_string(),
            name: def.name,
            status: ServiceStatus::Running,
            pid: Some(pid),
            port: def.default_port,
            version: None,
            error: None,
        };

        self.children.lock().unwrap().insert(
            id.to_string(),
            ActiveChild {
                child,
                pid,
                port: def.default_port,
            },
        );
        self.states.lock().unwrap().insert(id.to_string(), running_state.clone());

        Ok(running_state)
    }

    /// Stop a service by ID.
    pub fn stop_service(&self, id: &str) -> Result<ServiceState, String> {
        let mut child_opt = self.children.lock().unwrap().remove(id);
        if let Some(mut active) = child_opt.take() {
            // Update to Stopping
            {
                let mut states = self.states.lock().unwrap();
                if let Some(state) = states.get_mut(id) {
                    state.status = ServiceStatus::Stopping;
                }
            }

            // Graceful SIGTERM, wait up to 1 second
            let _ = terminate_process_tree(active.pid, false);
            let mut exited = false;
            for _ in 0..10 {
                if let Ok(Some(_)) = active.child.try_wait() {
                    exited = true;
                    break;
                }
                std::thread::sleep(Duration::from_millis(100));
            }

            // Force kill if still alive
            if !exited {
                let _ = terminate_process_tree(active.pid, true);
                let _ = active.child.wait();
            }
        }

        let def_name = self
            .definitions
            .lock()
            .unwrap()
            .get(id)
            .map(|d| d.name.clone())
            .unwrap_or_else(|| id.to_string());

        let stopped_state = ServiceState {
            id: id.to_string(),
            name: def_name,
            status: ServiceStatus::Stopped,
            pid: None,
            port: self.definitions.lock().unwrap().get(id).and_then(|d| d.default_port),
            version: None,
            error: None,
        };

        self.states.lock().unwrap().insert(id.to_string(), stopped_state.clone());
        Ok(stopped_state)
    }

    /// Restart a service by ID.
    pub fn restart_service(&self, id: &str) -> Result<ServiceState, String> {
        let _ = self.stop_service(id);
        std::thread::sleep(Duration::from_millis(200));
        self.start_service(id)
    }

    /// Start all registered services in order (respecting active web server).
    pub fn start_all_services(&self) -> Result<StackStatus, String> {
        let active_ws = self.get_active_web_server();
        let inactive_ws = if active_ws == "nginx" { "apache" } else { "nginx" };

        let order = ["dns", "mysql", "php", active_ws.as_str(), "mailpit"];
        let registered_ids: Vec<String> = self.definitions.lock().unwrap().keys().cloned().collect();

        for service_id in &order {
            if registered_ids.contains(&service_id.to_string()) {
                let _ = self.start_service(service_id);
            }
        }

        // Start any other services not in the predefined list, but skip inactive web server
        for service_id in &registered_ids {
            if service_id != inactive_ws && !order.contains(&service_id.as_str()) {
                let _ = self.start_service(service_id);
            }
        }

        Ok(self.get_stack_status())
    }

    /// Stop all registered services.
    pub fn stop_all_services(&self) -> Result<StackStatus, String> {
        let active_ids: Vec<String> = self.children.lock().unwrap().keys().cloned().collect();
        for id in active_ids {
            let _ = self.stop_service(&id);
        }
        Ok(self.get_stack_status())
    }

    fn probe_port_health(port: u16, max_retries: u32, delay_ms: u64) -> bool {
        let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
        for _ in 0..max_retries {
            if TcpStream::connect_timeout(&addr, Duration::from_millis(delay_ms)).is_ok() {
                return true;
            }
            std::thread::sleep(Duration::from_millis(delay_ms));
        }
        false
    }
}

impl Drop for ProcessSupervisor {
    fn drop(&mut self) {
        // Anti-zombie guarantee: terminate every active process upon drop
        let mut children = self.children.lock().unwrap();
        for (id, active) in children.iter_mut() {
            let _ = terminate_process_tree(active.pid, true);
            let _ = active.child.wait();
            eprintln!("Terminated child process for {} (PID {})", id, active.pid);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supervisor_lifecycle_with_mock_process() {
        let supervisor = ProcessSupervisor::new(PathBuf::from("/tmp"));

        let mock_def = ServiceDefinition {
            id: "mock_service".to_string(),
            name: "Mock Service".to_string(),
            binary_candidates: vec!["sleep".to_string(), "timeout".to_string()],
            default_port: None,
            args: vec!["10".to_string()],
            work_dir: None,
            env: HashMap::new(),
        };

        supervisor.register_service(mock_def);

        let initial_status = supervisor.get_stack_status();
        let state = initial_status.services.get("mock_service").unwrap();
        assert_eq!(state.status, ServiceStatus::Stopped);

        // Start
        let start_res = supervisor.start_service("mock_service");
        assert!(start_res.is_ok(), "Failed to start mock service: {:?}", start_res);
        let started_state = start_res.unwrap();
        assert_eq!(started_state.status, ServiceStatus::Running);
        assert!(started_state.pid.is_some());

        let running_pid = started_state.pid.unwrap();

        // Stop
        let stop_res = supervisor.stop_service("mock_service");
        assert!(stop_res.is_ok());
        let stopped_state = stop_res.unwrap();
        assert_eq!(stopped_state.status, ServiceStatus::Stopped);

        // Verify PID is dead
        #[cfg(unix)]
        {
            let res = unsafe { libc::kill(running_pid as i32, 0) };
            assert_ne!(res, 0, "Process is still alive after stop!");
        }
    }
}
