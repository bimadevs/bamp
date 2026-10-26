pub mod config;
pub mod dns;
pub mod inspector;
pub mod platform;
pub mod ssl;
pub mod supervisor;

use std::collections::HashMap;
use std::sync::Arc;
use supervisor::{ProcessSupervisor, ServiceDefinition, ServiceState, StackStatus};

#[tauri::command]
fn start_service(
    service_id: String,
    supervisor: tauri::State<'_, Arc<ProcessSupervisor>>,
) -> Result<ServiceState, String> {
    supervisor.start_service(&service_id)
}

#[tauri::command]
fn stop_service(
    service_id: String,
    supervisor: tauri::State<'_, Arc<ProcessSupervisor>>,
) -> Result<ServiceState, String> {
    supervisor.stop_service(&service_id)
}

#[tauri::command]
fn restart_service(
    service_id: String,
    supervisor: tauri::State<'_, Arc<ProcessSupervisor>>,
) -> Result<ServiceState, String> {
    supervisor.restart_service(&service_id)
}

#[tauri::command]
fn start_all_services(
    supervisor: tauri::State<'_, Arc<ProcessSupervisor>>,
) -> Result<StackStatus, String> {
    supervisor.start_all_services()
}

#[tauri::command]
fn stop_all_services(
    supervisor: tauri::State<'_, Arc<ProcessSupervisor>>,
) -> Result<StackStatus, String> {
    supervisor.stop_all_services()
}

#[tauri::command]
fn get_stack_status(
    supervisor: tauri::State<'_, Arc<ProcessSupervisor>>,
) -> Result<StackStatus, String> {
    Ok(supervisor.get_stack_status())
}

pub fn create_default_supervisor() -> Arc<ProcessSupervisor> {
    let base_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let supervisor = Arc::new(ProcessSupervisor::new(base_dir));

    // Register Nginx
    supervisor.register_service(ServiceDefinition {
        id: "nginx".to_string(),
        name: "Nginx Web Server".to_string(),
        binary_candidates: vec!["nginx".to_string()],
        default_port: Some(80),
        args: vec!["-p".to_string(), ".".to_string(), "-c".to_string(), "data/nginx/nginx.conf".to_string()],
        work_dir: None,
        env: HashMap::new(),
    });

    // Register Apache (httpd)
    supervisor.register_service(ServiceDefinition {
        id: "apache".to_string(),
        name: "Apache HTTP Server".to_string(),
        binary_candidates: vec!["httpd".to_string(), "apache2".to_string()],
        default_port: Some(80),
        args: vec!["-f".to_string(), "data/apache/httpd.conf".to_string(), "-DFOREGROUND".to_string()],
        work_dir: None,
        env: HashMap::new(),
    });

    // Register PHP-FPM
    supervisor.register_service(ServiceDefinition {
        id: "php".to_string(),
        name: "PHP-FPM".to_string(),
        binary_candidates: vec!["php-fpm".to_string(), "php-fpm8.3".to_string(), "php-fpm8.2".to_string()],
        default_port: Some(9000),
        args: vec!["--fpm-config".to_string(), "data/php/php-fpm.conf".to_string(), "-F".to_string()],
        work_dir: None,
        env: HashMap::new(),
    });

    // Register MySQL / MariaDB
    supervisor.register_service(ServiceDefinition {
        id: "mysql".to_string(),
        name: "MySQL / MariaDB".to_string(),
        binary_candidates: vec!["mariadbd".to_string(), "mysqld".to_string()],
        default_port: Some(3306),
        args: vec![
            "--defaults-file=data/mysql/my.cnf".to_string(),
            "--datadir=data/mysql/data".to_string(),
            "--socket=data/mysql/mysql.sock".to_string(),
            "--port=3306".to_string(),
        ],
        work_dir: None,
        env: HashMap::new(),
    });

    // Register Mailpit
    supervisor.register_service(ServiceDefinition {
        id: "mailpit".to_string(),
        name: "Mailpit SMTP & Web".to_string(),
        binary_candidates: vec!["mailpit".to_string()],
        default_port: Some(8025),
        args: vec![
            "--smtp".to_string(), "127.0.0.1:1025".to_string(),
            "--listen".to_string(), "127.0.0.1:8025".to_string(),
        ],
        work_dir: None,
        env: HashMap::new(),
    });

    supervisor
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let supervisor = create_default_supervisor();

    tauri::Builder::default()
        .manage(supervisor)
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            start_service,
            stop_service,
            restart_service,
            start_all_services,
            stop_all_services,
            get_stack_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
