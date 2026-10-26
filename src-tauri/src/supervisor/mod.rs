//! Process Supervisor for BAMP services (Nginx, PHP-FPM, MySQL, Mailpit).
//! Handles process lifecycle, health-checking, and anti-zombie lifetime management.

use serde::{Deserialize, Serialize};

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
    pub services: std::collections::HashMap<String, ServiceState>,
}
