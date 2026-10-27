pub mod config;
pub mod dns;
pub mod inspector;
pub mod php;
pub mod platform;
pub mod ssl;
pub mod supervisor;
pub mod tools;

use std::collections::HashMap;
use std::sync::Arc;
use supervisor::{ProcessSupervisor, ServiceDefinition, ServiceState, StackStatus};
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;
use tauri::Manager;

/// Resolves the base root directory for BAMP, checking both working directory and executable directory
pub fn resolve_bamp_base_dir() -> std::path::PathBuf {
    if let Ok(cwd) = std::env::current_dir() {
        if cwd.join("bin").exists() && cwd.join("templates").exists() {
            return cwd;
        }
    }

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            if exe_dir.join("bin").exists() || exe_dir.join("templates").exists() {
                return exe_dir.to_path_buf();
            }
        }
    }

    std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
}

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
    let base_dir = resolve_bamp_base_dir();
    let ssl_manager = ssl::SslManager::new(&base_dir);
    ssl_manager.ensure_certificates()
}

#[tauri::command]
fn install_root_ca() -> Result<String, String> {
    let base_dir = resolve_bamp_base_dir();
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

#[tauri::command]
fn get_available_php_versions(
    config_manager: tauri::State<'_, Arc<config::ConfigManager>>,
) -> Result<Vec<String>, String> {
    Ok(config_manager.get_available_php_versions())
}

#[tauri::command]
fn set_site_php_version(
    site_id: String,
    php_version: String,
    config_manager: tauri::State<'_, Arc<config::ConfigManager>>,
) -> Result<config::Site, String> {
    config_manager.set_site_php_version(&site_id, &php_version)
}

#[tauri::command]
fn get_active_web_server(
    supervisor: tauri::State<'_, Arc<ProcessSupervisor>>,
) -> Result<String, String> {
    Ok(supervisor.get_active_web_server())
}

#[tauri::command]
fn set_active_web_server(
    server_id: String,
    config_manager: tauri::State<'_, Arc<config::ConfigManager>>,
) -> Result<String, String> {
    config_manager.set_active_web_server(&server_id)
}

#[tauri::command]
fn reload_web_server(
    supervisor: tauri::State<'_, Arc<ProcessSupervisor>>,
) -> Result<(), String> {
    supervisor.reload_web_server()
}

#[tauri::command]
fn get_settings(
    config_manager: tauri::State<'_, Arc<config::ConfigManager>>,
) -> Result<config::AppSettings, String> {
    Ok(config_manager.get_settings())
}

#[tauri::command]
fn get_php_config() -> Result<php::PhpConfig, String> {
    let base_dir = resolve_bamp_base_dir();
    php::PhpManager::get_php_config(&base_dir)
}

#[tauri::command]
fn toggle_php_extension(
    extension_name: String,
    enabled: bool,
    supervisor: tauri::State<'_, Arc<ProcessSupervisor>>,
) -> Result<bool, String> {
    let base_dir = resolve_bamp_base_dir();
    let res = php::PhpManager::toggle_extension(&base_dir, &extension_name, enabled)?;
    let _ = supervisor.restart_service("php");
    Ok(res)
}

#[tauri::command]
fn update_php_directive(
    key: String,
    value: String,
    supervisor: tauri::State<'_, Arc<ProcessSupervisor>>,
) -> Result<String, String> {
    let base_dir = resolve_bamp_base_dir();
    let res = php::PhpManager::update_directive(&base_dir, &key, &value)?;
    let _ = supervisor.restart_service("php");
    Ok(res)
}

#[tauri::command]
fn launch_isolated_terminal(
    active_php: Option<String>,
) -> Result<String, String> {
    let base_dir = resolve_bamp_base_dir();
    let php_ver = active_php.unwrap_or_else(|| "default".to_string());
    tools::ToolsManager::launch_isolated_terminal(&base_dir, &php_ver)
}

#[tauri::command]
fn launch_database_client() -> Result<String, String> {
    let base_dir = resolve_bamp_base_dir();
    tools::ToolsManager::launch_database_client(&base_dir)
}

#[tauri::command]
fn scaffold_app(
    app_type: String,
    project_name: String,
    config_manager: tauri::State<'_, Arc<config::ConfigManager>>,
) -> Result<String, String> {
    let base_dir = resolve_bamp_base_dir();
    let url = tools::ToolsManager::scaffold_app(&base_dir, &app_type, &project_name)?;
    let _ = config_manager.render_all_configs();
    Ok(url)
}

#[tauri::command]
fn get_package_hub() -> Result<Vec<tools::PackageInfo>, String> {
    let base_dir = resolve_bamp_base_dir();
    Ok(tools::ToolsManager::get_package_hub(&base_dir))
}

#[tauri::command]
fn install_package(package_id: String) -> Result<String, String> {
    let base_dir = resolve_bamp_base_dir();
    tools::ToolsManager::install_package(&base_dir, &package_id)
}

#[tauri::command]
fn download_service_binaries(services: Option<String>) -> Result<String, String> {
    let base_dir = resolve_bamp_base_dir();
    let target_services = services.unwrap_or_else(|| "all".to_string());
    tools::ToolsManager::download_service_binaries(&base_dir, &target_services)
}

pub fn create_default_supervisor() -> Arc<ProcessSupervisor> {
    let base_dir = resolve_bamp_base_dir();
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

    // Register PHP-FPM / FastCGI default
    supervisor.register_service(ServiceDefinition {
        id: "php".to_string(),
        name: "PHP (FastCGI / FPM)".to_string(),
        binary_candidates: vec![
            "php-cgi".to_string(),
            "php-cgi8.3".to_string(),
            "php".to_string(),
            "php-fpm".to_string(),
            "php-fpm8.3".to_string(),
            "php-fpm8.2".to_string(),
        ],
        default_port: Some(9000),
        args: if cfg!(windows) {
            vec!["-b".to_string(), "127.0.0.1:9000".to_string()]
        } else {
            vec!["--fpm-config".to_string(), "data/php/php-fpm.conf".to_string(), "-F".to_string()]
        },
        work_dir: None,
        env: {
            let mut env = HashMap::new();
            if cfg!(windows) {
                env.insert("PHP_FCGI_MAX_REQUESTS".to_string(), "0".to_string());
            }
            env
        },
    });

    // Register MySQL / MariaDB
    supervisor.register_service(ServiceDefinition {
        id: "mysql".to_string(),
        name: "MySQL / MariaDB".to_string(),
        binary_candidates: vec!["mariadbd".to_string(), "mysqld".to_string()],
        default_port: Some(3306),
        args: {
            let mut args = vec![
                "--defaults-file=data/mysql/my.cnf".to_string(),
                "--datadir=data/mysql/data".to_string(),
                "--socket=data/mysql/mysql.sock".to_string(),
                "--port=3306".to_string(),
            ];
            if cfg!(windows) {
                args.push("--console".to_string());
            }
            args
        },
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
    let base_dir = resolve_bamp_base_dir();
    let config_manager = Arc::new(config::ConfigManager::new(base_dir));

    // Connect supervisor to config manager
    config_manager.set_supervisor(supervisor.clone());
    let saved_settings = config_manager.get_settings();
    let _ = supervisor.set_active_web_server(&saved_settings.active_web_server);

    // Render initial configurations
    let _ = config_manager.render_all_configs();

    // Start background watcher on www/
    let watcher_cm = config_manager.clone();
    let _ = watcher_cm.start_watcher();

    tauri::Builder::default()
        .manage(supervisor.clone())
        .manage(config_manager)
        .plugin(tauri_plugin_opener::init())
        .setup(move |app| {
            supervisor.set_app_handle(app.handle().clone());

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
            render_configs,
            get_available_php_versions,
            set_site_php_version,
            get_active_web_server,
            set_active_web_server,
            reload_web_server,
            get_settings,
            get_php_config,
            toggle_php_extension,
            update_php_directive,
            launch_isolated_terminal,
            launch_database_client,
            scaffold_app,
            get_package_hub,
            install_package,
            download_service_binaries
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
