pub mod config;
pub mod dns;
pub mod inspector;
pub mod platform;
pub mod ssl;
pub mod supervisor;

use std::collections::HashMap;
use std::sync::Arc;
use supervisor::{ProcessSupervisor, ServiceDefinition, ServiceState, StackStatus};
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;
use tauri::Manager;

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

#[tauri::command]
fn sync_hosts(domains: Vec<String>) -> Result<(), String> {
    let domain_refs: Vec<&str> = domains.iter().map(|s| s.as_str()).collect();
    dns::HostsManager::sync_domains(&domain_refs)
}

#[tauri::command]
fn ensure_ssl_certificates() -> Result<ssl::SslPaths, String> {
    let base_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let ssl_manager = ssl::SslManager::new(&base_dir);
    ssl_manager.ensure_certificates()
}

#[tauri::command]
fn install_root_ca() -> Result<String, String> {
    let base_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let ssl_manager = ssl::SslManager::new(&base_dir);
    ssl_manager.install_root_ca()
}

#[tauri::command]
fn get_sites(config_manager: tauri::State<'_, Arc<config::ConfigManager>>) -> Result<Vec<config::Site>, String> {
    config_manager.refresh_sites()
}

#[tauri::command]
fn add_linked_folder(
    path: String,
    config_manager: tauri::State<'_, Arc<config::ConfigManager>>,
) -> Result<Vec<config::Site>, String> {
    config_manager.add_linked_folder(std::path::PathBuf::from(path))
}

#[tauri::command]
fn render_configs(config_manager: tauri::State<'_, Arc<config::ConfigManager>>) -> Result<(), String> {
    config_manager.render_all_configs()
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
    let base_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let config_manager = Arc::new(config::ConfigManager::new(base_dir));

    // Render initial configurations
    let _ = config_manager.render_all_configs();

    // Start background watcher on www/
    let watcher_cm = config_manager.clone();
    let _ = watcher_cm.start_watcher();

    tauri::Builder::default()
        .manage(supervisor)
        .manage(config_manager)
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let dns_server = dns::DnsServer::new();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = dns_server.start(53).await {
                    eprintln!("DNS Server Warning: {}", e);
                }
            });

            // Build System Tray Menu
            let open_dash = MenuItemBuilder::with_id("open_dash", "Open BAMP Dashboard").build(app)?;
            let open_www = MenuItemBuilder::with_id("open_www", "Open www folder").build(app)?;
            let start_all = MenuItemBuilder::with_id("start_all", "Start All Services").build(app)?;
            let stop_all = MenuItemBuilder::with_id("stop_all", "Stop All Services").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "Quit BAMP").build(app)?;

            let tray_menu = MenuBuilder::new(app)
                .items(&[&open_dash, &open_www, &start_all, &stop_all, &quit])
                .build()?;

            let mut tray_builder = TrayIconBuilder::new()
                .menu(&tray_menu)
                .show_menu_on_left_click(false)
                .on_menu_event(move |app_handle, event| match event.id().as_ref() {
                    "open_dash" => {
                        if let Some(w) = app_handle.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    "open_www" => {
                        let base = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
                        let www = base.join("www");
                        let _ = std::process::Command::new("xdg-open").arg(www).spawn();
                    }
                    "start_all" => {
                        if let Some(s) = app_handle.try_state::<Arc<ProcessSupervisor>>() {
                            let _ = s.start_all_services();
                        }
                    }
                    "stop_all" => {
                        if let Some(s) = app_handle.try_state::<Arc<ProcessSupervisor>>() {
                            let _ = s.stop_all_services();
                        }
                    }
                    "quit" => {
                        app_handle.exit(0);
                    }
                    _ => (),
                });

            if let Some(icon) = app.default_window_icon() {
                tray_builder = tray_builder.icon(icon.clone());
            }

            let _tray = tray_builder.build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_service,
            stop_service,
            restart_service,
            start_all_services,
            stop_all_services,
            get_stack_status,
            sync_hosts,
            ensure_ssl_certificates,
            install_root_ca,
            get_sites,
            add_linked_folder,
            render_configs
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
