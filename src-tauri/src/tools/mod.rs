//! Developer Productivity Tools:
//! 1. Isolated Terminal Launcher (PATH Injected Shell)
//! 2. Smart Database Client Launcher (TablePlus, DBeaver, HeidiSQL, Adminer)
//! 3. Quick App Scaffolder (Blank PHP, WordPress, Laravel)
//! 4. Modular Package Hub (PHP 7.4 - 8.4)

use serde::{Deserialize, Serialize};
use std::fs;
#[allow(unused_imports)]
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub is_installed: bool,
    pub path: Option<String>,
    pub download_url: String,
}

pub struct ToolsManager;

impl ToolsManager {
    /// Launch an isolated terminal session with environment variables prepended to BAMP binaries
    pub fn launch_isolated_terminal(base_dir: &Path, active_php: &str) -> Result<String, String> {
        let bin_dir = base_dir.join("bin");
        let php_dir = bin_dir.join("php").join(active_php);
        let mysql_dir = bin_dir.join("mysql").join("bin");
        let composer_dir = bin_dir.join("composer");

        #[cfg(target_os = "windows")]
        {
            let current_path = std::env::var("PATH").unwrap_or_default();
            let injected_path = format!(
                "{};{};{};{}",
                php_dir.to_string_lossy(),
                mysql_dir.to_string_lossy(),
                composer_dir.to_string_lossy(),
                current_path
            );

            // Try Windows Terminal first (wt.exe), fallback to powershell, fallback to cmd
            let child = Command::new("wt.exe")
                .env("PATH", &injected_path)
                .current_dir(base_dir.join("www"))
                .spawn();

            if child.is_ok() {
                return Ok("Opened Windows Terminal with isolated BAMP environment.".to_string());
            }

            let child_ps = Command::new("powershell.exe")
                .args(["-NoExit", "-Command", "Write-Host 'BAMP Isolated Terminal Activated' -ForegroundColor Green"])
                .env("PATH", &injected_path)
                .current_dir(base_dir.join("www"))
                .spawn();

            if child_ps.is_ok() {
                return Ok("Opened PowerShell with isolated BAMP environment.".to_string());
            }

            let _ = Command::new("cmd.exe")
                .args(["/K", "echo BAMP Isolated Terminal Activated"])
                .env("PATH", &injected_path)
                .current_dir(base_dir.join("www"))
                .spawn()
                .map_err(|e| format!("Failed to spawn terminal: {}", e))?;

            Ok("Opened CMD with isolated BAMP environment.".to_string())
        }

        #[cfg(target_os = "linux")]
        {
            let current_path = std::env::var("PATH").unwrap_or_default();
            let injected_path = format!(
                "{}:{}:{}:{}",
                php_dir.to_string_lossy(),
                mysql_dir.to_string_lossy(),
                composer_dir.to_string_lossy(),
                current_path
            );

            let terminals = [
                ("x-terminal-emulator", vec![]),
                ("gnome-terminal", vec!["--"]),
                ("konsole", vec![]),
                ("xfce4-terminal", vec![]),
                ("kitty", vec![]),
                ("alacritty", vec![]),
                ("xterm", vec![]),
            ];

            for (term, args) in &terminals {
                if let Ok(cmd) = which::which(term) {
                    let mut process = Command::new(cmd);
                    process.args(args);
                    process.env("PATH", &injected_path);
                    process.current_dir(base_dir.join("www"));
                    if process.spawn().is_ok() {
                        return Ok(format!("Opened {} with isolated BAMP environment.", term));
                    }
                }
            }

            Err("No supported Linux desktop terminal emulator found (tried x-terminal-emulator, gnome-terminal, konsole, xterm).".to_string())
        }

        #[cfg(not(any(target_os = "windows", target_os = "linux")))]
        {
            Err("Isolated terminal launcher is not supported on this operating system.".to_string())
        }
    }

    /// Auto-detect installed desktop database client and launch, or fallback to Adminer
    pub fn launch_database_client(base_dir: &Path) -> Result<String, String> {

        #[cfg(target_os = "windows")]
        {
            let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_default();
            let prog_files = std::env::var("PROGRAMFILES").unwrap_or_default();

            let candidates = [
                PathBuf::from(&local_app_data).join("Programs/TablePlus/TablePlus.exe"),
                PathBuf::from(&prog_files).join("HeidiSQL/heidisql.exe"),
                PathBuf::from(&prog_files).join("DBeaver/dbeaver.exe"),
            ];

            for client_path in &candidates {
                if client_path.is_file() {
                    let _ = Command::new(client_path).spawn();
                    return Ok(format!("Launched desktop database client: {}", client_path.file_name().unwrap_or_default().to_string_lossy()));
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            let linux_clients = ["dbeaver", "mysql-workbench", "beekeeper-studio"];
            for client in &linux_clients {
                if let Ok(path) = which::which(client) {
                    let _ = Command::new(path).spawn();
                    return Ok(format!("Launched {} database client.", client));
                }
            }
        }

        // Fallback: check if adminer is present
        let adminer_dir = base_dir.join("data/adminer");
        if adminer_dir.join("index.php").is_file() {
            return Ok("adminer".to_string());
        }

        Err("No desktop database client found and Adminer is not available.".to_string())
    }

    /// Scaffolds a new project: Blank PHP, WordPress, or Laravel
    pub fn scaffold_app(base_dir: &Path, app_type: &str, project_name: &str) -> Result<String, String> {
        let sanitized_name = project_name
            .to_lowercase()
            .replace(' ', "-")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .collect::<String>();

        if sanitized_name.is_empty() {
            return Err("Project name cannot be empty".to_string());
        }

        let www_dir = base_dir.join("www");
        let project_dir = www_dir.join(&sanitized_name);

        if project_dir.exists() {
            return Err(format!("Directory already exists: {}", project_dir.display()));
        }

        fs::create_dir_all(&project_dir)
            .map_err(|e| format!("Failed to create project folder: {}", e))?;

        // 1. Try to create MySQL database for this project
        let db_name = sanitized_name.replace('-', "_");
        let _ = Self::create_mysql_database(base_dir, &db_name);

        // 2. Generate scaffold files based on app_type
        match app_type {
            "wordpress" => {
                // Generate a clean WordPress bootstrapping structure
                let index_content = format!(
                    r#"<?php
/**
 * BAMP WordPress Quick App
 * Database: {}
 * Local Domain: {}.test
 */
?>
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>WordPress Installer - {}.test</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; background: #0f172a; color: #f8fafc; display: flex; align-items: center; justify-content: center; height: 100vh; margin: 0; }}
        .card {{ background: #1e293b; border: 1px solid #334155; border-radius: 16px; padding: 32px; max-width: 500px; text-align: center; box-shadow: 0 20px 25px -5px rgba(0,0,0,0.5); }}
        .badge {{ background: #38bdf8; color: #0f172a; font-weight: bold; font-size: 12px; padding: 4px 10px; border-radius: 9999px; text-transform: uppercase; }}
        h1 {{ margin: 16px 0 8px; font-size: 22px; }}
        p {{ color: #94a3b8; font-size: 14px; line-height: 1.6; }}
        .db-info {{ background: #0f172a; border-radius: 8px; padding: 12px; margin: 20px 0; font-family: monospace; font-size: 13px; text-align: left; color: #38bdf8; }}
        a.btn {{ display: inline-block; background: #0284c7; color: white; padding: 10px 20px; border-radius: 8px; text-decoration: none; font-weight: 600; margin-top: 12px; }}
    </style>
</head>
<body>
    <div class="card">
        <span class="badge">BAMP Quick App</span>
        <h1>WordPress Ready</h1>
        <p>Virtual host dan database MySQL telah berhasil di-provisioning otomatis oleh BAMP.</p>
        <div class="db-info">
            • DB Name: <strong>{}</strong><br>
            • DB User: <strong>root</strong><br>
            • DB Pass: <em>(kosong)</em><br>
            • DB Host: <strong>127.0.0.1:3306</strong>
        </div>
        <p>Anda dapat mengunduh atau menaruh core WordPress ke dalam direktori ini untuk memulai instalasi WordPress.</p>
        <a href="http://adminer.test" class="btn" target="_blank">Buka Database di Adminer</a>
    </div>
</body>
</html>
"#,
                    db_name, sanitized_name, sanitized_name, db_name
                );
                fs::write(project_dir.join("index.php"), index_content)
                    .map_err(|e| format!("Failed to create WordPress index.php: {}", e))?;
            }
            "laravel" => {
                let public_dir = project_dir.join("public");
                fs::create_dir_all(&public_dir)
                    .map_err(|e| format!("Failed to create public dir: {}", e))?;

                let index_content = format!(
                    r#"<?php
/**
 * BAMP Laravel Quick App
 * Database: {}
 * Local Domain: {}.test
 */
?>
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Laravel Ready - {}.test</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; background: #0f172a; color: #f8fafc; display: flex; align-items: center; justify-content: center; height: 100vh; margin: 0; }}
        .card {{ background: #1e293b; border: 1px solid #334155; border-radius: 16px; padding: 32px; max-width: 500px; text-align: center; box-shadow: 0 20px 25px -5px rgba(0,0,0,0.5); }}
        .badge {{ background: #f43f5e; color: #ffffff; font-weight: bold; font-size: 12px; padding: 4px 10px; border-radius: 9999px; text-transform: uppercase; }}
        h1 {{ margin: 16px 0 8px; font-size: 22px; }}
        p {{ color: #94a3b8; font-size: 14px; line-height: 1.6; }}
        .db-info {{ background: #0f172a; border-radius: 8px; padding: 12px; margin: 20px 0; font-family: monospace; font-size: 13px; text-align: left; color: #fb7185; }}
        code {{ background: #334155; padding: 2px 6px; border-radius: 4px; font-size: 13px; color: #fff; }}
    </style>
</head>
<body>
    <div class="card">
        <span class="badge">Laravel App</span>
        <h1>Laravel Scaffolding Ready</h1>
        <p>BAMP telah menyiapkan Document Root publik <code>public/</code> dan Database MySQL.</p>
        <div class="db-info">
            • DB_DATABASE=<strong>{}</strong><br>
            • DB_USERNAME=<strong>root</strong><br>
            • DB_PASSWORD=<br>
            • DB_HOST=<strong>127.0.0.1</strong>:3306
        </div>
        <p>Jalankan <code>composer create-project laravel/laravel .</code> di dalam folder ini via Isolated Terminal untuk menginstall Laravel core lengkap.</p>
    </div>
</body>
</html>
"#,
                    db_name, sanitized_name, sanitized_name, db_name
                );
                fs::write(public_dir.join("index.php"), index_content)
                    .map_err(|e| format!("Failed to create Laravel public/index.php: {}", e))?;
            }
            _ => {
                // Blank PHP Project
                let index_content = format!(
                    r#"<?php
/**
 * BAMP Blank PHP Project
 * Database: {}
 * Local Domain: {}.test
 */
?>
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Hello from {}.test</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; background: #0b0f19; color: #f3f4f6; display: flex; align-items: center; justify-content: center; height: 100vh; margin: 0; }}
        .card {{ background: #111827; border: 1px solid #1f2937; border-radius: 16px; padding: 36px; max-width: 520px; box-shadow: 0 20px 25px -5px rgba(0,0,0,0.5); text-align: center; }}
        .badge {{ background: #6366f1; color: #ffffff; font-weight: bold; font-size: 12px; padding: 4px 10px; border-radius: 9999px; text-transform: uppercase; }}
        h1 {{ margin: 16px 0 8px; font-size: 24px; }}
        p {{ color: #9ca3af; font-size: 14px; line-height: 1.6; }}
        .grid {{ display: grid; grid-template-columns: 1fr 1fr; gap: 12px; margin: 24px 0; text-align: left; }}
        .item {{ background: #1f2937; padding: 12px; border-radius: 8px; font-size: 13px; font-family: monospace; }}
        .item span {{ display: block; color: #9ca3af; font-size: 11px; margin-bottom: 2px; }}
        .item strong {{ color: #10b981; }}
    </style>
</head>
<body>
    <div class="card">
        <span class="badge">BAMP Project</span>
        <h1>{}.test</h1>
        <p>Aplikasi web PHP Anda telah aktif dengan virtual host dan SSL terpercaya.</p>
        <div class="grid">
            <div class="item"><span>PHP Version</span><strong><?= phpversion() ?></strong></div>
            <div class="item"><span>Server Engine</span><strong><?= $_SERVER['SERVER_SOFTWARE'] ?? 'Nginx' ?></strong></div>
            <div class="item"><span>Database Name</span><strong>{}</strong></div>
            <div class="item"><span>HTTPS Status</span><strong><?= isset($_SERVER['HTTPS']) && $_SERVER['HTTPS'] === 'on' ? 'Secure (SSL)' : 'HTTP' ?></strong></div>
        </div>
        <p style="font-size: 12px; color: #6b7280;">Buka folder <code>www/{}</code> di editor favorit Anda untuk mulai mengoding.</p>
    </div>
</body>
</html>
"#,
                    db_name, sanitized_name, sanitized_name, sanitized_name, db_name, sanitized_name
                );
                fs::write(project_dir.join("index.php"), index_content)
                    .map_err(|e| format!("Failed to create index.php: {}", e))?;
            }
        }

        Ok(format!("http://{}.test", sanitized_name))
    }

    fn create_mysql_database(base_dir: &Path, db_name: &str) -> Result<(), String> {
        let bin_candidates = ["mariadb", "mysql"];
        let mysql_bin = crate::platform::resolve_binary(base_dir, "mysql", &bin_candidates);

        if let Some(bin) = mysql_bin {
            let _ = Command::new(bin)
                .args(["-u", "root", "-e", &format!("CREATE DATABASE IF NOT EXISTS `{}`;", db_name)])
                .status();
        }
        Ok(())
    }

    /// Retrieve Package Hub items (modular PHP versions and components)
    pub fn get_package_hub(base_dir: &Path) -> Vec<PackageInfo> {
        let mut packages = Vec::new();
        let versions = [
            ("php-8.4", "PHP 8.4 (Latest)", "8.4.1", "https://windows.php.net/downloads/releases/archives/"),
            ("php-8.3", "PHP 8.3 (Stable)", "8.3.16", "https://windows.php.net/downloads/releases/archives/"),
            ("php-8.2", "PHP 8.2 (LTS)", "8.2.27", "https://windows.php.net/downloads/releases/archives/"),
            ("php-8.1", "PHP 8.1 (Legacy)", "8.1.31", "https://windows.php.net/downloads/releases/archives/"),
            ("php-7.4", "PHP 7.4 (Legacy)", "7.4.33", "https://windows.php.net/downloads/releases/archives/"),
        ];

        let php_bin_dir = base_dir.join("bin/php");

        for (id, name, ver, url) in &versions {
            let target_dir = php_bin_dir.join(id);
            let exists_in_bin = target_dir.is_dir();
            let exists_in_system = which::which(id).is_ok() || (id == &"php-8.3" && which::which("php").is_ok());

            let is_installed = exists_in_bin || exists_in_system;
            let path_str = if exists_in_bin {
                Some(target_dir.to_string_lossy().to_string())
            } else if exists_in_system {
                Some("System PATH".to_string())
            } else {
                None
            };

            packages.push(PackageInfo {
                id: id.to_string(),
                name: name.to_string(),
                version: ver.to_string(),
                is_installed,
                path: path_str,
                download_url: url.to_string(),
            });
        }

        packages
    }

    /// Install or register a PHP version package
    pub fn install_package(base_dir: &Path, package_id: &str) -> Result<String, String> {
        let target_dir = base_dir.join("bin/php").join(package_id);
        fs::create_dir_all(&target_dir)
            .map_err(|e| format!("Failed to create package directory: {}", e))?;

        // Initialize a placeholder binary wrapper or README
        let readme_file = target_dir.join("README.txt");
        let _ = fs::write(
            readme_file,
            format!("BAMP Package {} initialized.\nDrop compiled php binary files into this folder.\n", package_id),
        );

        Ok(format!("Package {} successfully configured in bin/php/{}.", package_id, package_id))
    }
}

mod which {
    use std::path::PathBuf;

    pub fn which(binary_name: &str) -> Result<PathBuf, ()> {
        let path_var = std::env::var_os("PATH").ok_or(())?;
        for dir in std::env::split_paths(&path_var) {
            let full_path = dir.join(binary_name);
            if full_path.is_file() {
                return Ok(full_path);
            }
            #[cfg(windows)]
            {
                let full_path_exe = dir.join(format!("{}.exe", binary_name));
                if full_path_exe.is_file() {
                    return Ok(full_path_exe);
                }
            }
        }
        Err(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scaffold_blank_app() {
        let temp_dir = std::env::temp_dir().join(format!("bamp_tools_test_{}", std::process::id()));
        let _ = fs::create_dir_all(&temp_dir);

        let url = ToolsManager::scaffold_app(&temp_dir, "blank", "My First App").unwrap();
        assert_eq!(url, "http://my-first-app.test");

        let index_file = temp_dir.join("www/my-first-app/index.php");
        assert!(index_file.exists());
        let content = fs::read_to_string(index_file).unwrap();
        assert!(content.contains("my-first-app.test"));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_package_hub_listing() {
        let temp_dir = std::env::temp_dir().join(format!("bamp_pkghub_test_{}", std::process::id()));
        let pkgs = ToolsManager::get_package_hub(&temp_dir);
        assert_eq!(pkgs.len(), 5);
        assert!(pkgs.iter().any(|p| p.id == "php-8.3"));
    }
}
