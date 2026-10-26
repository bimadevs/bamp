//! Configuration template rendering and dynamic virtual host generation.
//! Watches the `www/` directory and synchronizes web server virtual host configs.

use notify::{Event, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Site {
    pub id: String,
    pub name: String,
    pub domain: String,
    pub path: String,
    pub document_root: String,
    pub php_version: String,
    pub has_ssl: bool,
    pub status: String,
}

pub struct ConfigManager {
    base_dir: PathBuf,
    linked_folders: Mutex<Vec<PathBuf>>,
    cached_sites: Mutex<Vec<Site>>,
}

impl ConfigManager {
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            base_dir,
            linked_folders: Mutex::new(Vec::new()),
            cached_sites: Mutex::new(Vec::new()),
        }
    }

    pub fn add_linked_folder(&self, path: PathBuf) -> Result<Vec<Site>, String> {
        if !path.is_dir() {
            return Err(format!("Path is not a valid directory: {}", path.display()));
        }
        {
            let mut linked = self.linked_folders.lock().unwrap();
            if !linked.contains(&path) {
                linked.push(path);
            }
        }
        self.refresh_sites()
    }

    /// Scan `www/` and all linked folders to detect active project sites.
    pub fn refresh_sites(&self) -> Result<Vec<Site>, String> {
        let www_dir = self.base_dir.join("www");
        let _ = fs::create_dir_all(&www_dir);

        let mut sites = Vec::new();
        let mut scan_dirs = vec![www_dir];
        {
            let linked = self.linked_folders.lock().unwrap();
            scan_dirs.extend(linked.iter().cloned());
        }

        let cert_exists = self.base_dir.join("data/ssl/cert.pem").exists();

        for root_dir in scan_dirs {
            if let Ok(entries) = fs::read_dir(&root_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        let folder_name = entry.file_name().to_string_lossy().to_string();
                        // Ignore hidden folders
                        if folder_name.starts_with('.') {
                            continue;
                        }

                        let sanitized_domain = format!("{}.test", folder_name.to_lowercase().replace(' ', "-"));

                        // Determine document root
                        let public_candidate = path.join("public");
                        let doc_root = if public_candidate.is_dir()
                            && (public_candidate.join("index.php").is_file()
                                || public_candidate.join("index.html").is_file())
                        {
                            public_candidate
                        } else {
                            path.clone()
                        };

                        let site = Site {
                            id: format!("site_{}", folder_name),
                            name: folder_name,
                            domain: sanitized_domain,
                            path: path.to_string_lossy().to_string(),
                            document_root: doc_root.to_string_lossy().to_string(),
                            php_version: "default".to_string(),
                            has_ssl: cert_exists,
                            status: "active".to_string(),
                        };
                        sites.push(site);
                    }
                }
            }
        }

        // Sort alphabetically by domain
        sites.sort_by(|a, b| a.domain.cmp(&b.domain));

        *self.cached_sites.lock().unwrap() = sites.clone();
        Ok(sites)
    }

    pub fn get_sites(&self) -> Vec<Site> {
        self.cached_sites.lock().unwrap().clone()
    }

    /// Render base configs for Nginx, Apache, PHP-FPM, MySQL, and dynamic vhosts.
    pub fn render_all_configs(&self) -> Result<(), String> {
        let bamp_root = self.base_dir.to_string_lossy().to_string();
        let data_dir = self.base_dir.join("data");

        // Prepare data directories
        let _ = fs::create_dir_all(data_dir.join("nginx/vhosts"));
        let _ = fs::create_dir_all(data_dir.join("apache/vhosts"));
        let _ = fs::create_dir_all(data_dir.join("php"));
        let _ = fs::create_dir_all(data_dir.join("mysql/data"));
        let _ = fs::create_dir_all(data_dir.join("mysql/tmp"));
        let _ = fs::create_dir_all(data_dir.join("logs"));
        let _ = fs::create_dir_all(data_dir.join("ssl"));

        let mut replacements = HashMap::new();
        replacements.insert("bamp_root", bamp_root.clone());
        replacements.insert("http_port", "80".to_string());
        replacements.insert("https_port", "443".to_string());
        replacements.insert("fastcgi_pass", "127.0.0.1:9000".to_string());
        replacements.insert("mysql_port", "3306".to_string());
        replacements.insert("ssl_cert", self.base_dir.join("data/ssl/cert.pem").to_string_lossy().to_string());
        replacements.insert("ssl_key", self.base_dir.join("data/ssl/key.pem").to_string_lossy().to_string());

        // 1. Render nginx.conf
        let nginx_tpl = self.base_dir.join("templates/nginx/nginx.conf.template");
        if nginx_tpl.is_file() {
            let rendered = render_template(&nginx_tpl, &replacements)?;
            fs::write(data_dir.join("nginx/nginx.conf"), rendered)
                .map_err(|e| format!("Failed writing nginx.conf: {}", e))?;
        }

        // 2. Render httpd.conf
        let apache_tpl = self.base_dir.join("templates/apache/httpd.conf.template");
        if apache_tpl.is_file() {
            let rendered = render_template(&apache_tpl, &replacements)?;
            fs::write(data_dir.join("apache/httpd.conf"), rendered)
                .map_err(|e| format!("Failed writing httpd.conf: {}", e))?;
        }

        // 3. Render php-fpm.conf
        let php_tpl = self.base_dir.join("templates/php/php-fpm.conf.template");
        if php_tpl.is_file() {
            let rendered = render_template(&php_tpl, &replacements)?;
            fs::write(data_dir.join("php/php-fpm.conf"), rendered)
                .map_err(|e| format!("Failed writing php-fpm.conf: {}", e))?;
        }

        // 4. Render my.cnf
        let mysql_tpl = self.base_dir.join("templates/mysql/my.cnf.template");
        if mysql_tpl.is_file() {
            let rendered = render_template(&mysql_tpl, &replacements)?;
            fs::write(data_dir.join("mysql/my.cnf"), rendered)
                .map_err(|e| format!("Failed writing my.cnf: {}", e))?;
        }

        // 5. Render Virtual Hosts for all detected sites
        let sites = self.refresh_sites()?;

        // Clean existing vhost files first
        clean_directory(&data_dir.join("nginx/vhosts"));
        clean_directory(&data_dir.join("apache/vhosts"));

        let nginx_vhost_tpl = self.base_dir.join("templates/nginx/vhost.conf.template");
        let apache_vhost_tpl = self.base_dir.join("templates/apache/vhost.conf.template");

        for site in &sites {
            let mut site_replacements = replacements.clone();
            site_replacements.insert("domain", site.domain.clone());
            site_replacements.insert("document_root", site.document_root.clone());

            // Nginx vhost
            if nginx_vhost_tpl.is_file() {
                let rendered = render_template(&nginx_vhost_tpl, &site_replacements)?;
                let out_file = data_dir.join(format!("nginx/vhosts/{}.conf", site.domain));
                let _ = fs::write(out_file, rendered);
            }

            // Apache vhost
            if apache_vhost_tpl.is_file() {
                let rendered = render_template(&apache_vhost_tpl, &site_replacements)?;
                let out_file = data_dir.join(format!("apache/vhosts/{}.conf", site.domain));
                let _ = fs::write(out_file, rendered);
            }
        }

        Ok(())
    }

    /// Start background directory watcher on `www/`.
    pub fn start_watcher(self: Arc<Self>) -> Result<(), String> {
        let www_dir = self.base_dir.join("www");
        let _ = fs::create_dir_all(&www_dir);

        let manager_clone = self.clone();

        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                // If a path was created, removed, or modified
                if event.kind.is_create() || event.kind.is_remove() || event.kind.is_modify() {
                    let _ = manager_clone.render_all_configs();
                }
            }
        })
        .map_err(|e| format!("Failed to create filesystem watcher: {}", e))?;

        watcher
            .watch(&www_dir, RecursiveMode::NonRecursive)
            .map_err(|e| format!("Failed to watch www directory: {}", e))?;

        // Keep watcher alive in thread
        std::thread::spawn(move || {
            let _keep_alive = watcher;
            loop {
                std::thread::sleep(std::time::Duration::from_secs(3600));
            }
        });

        Ok(())
    }
}

fn render_template(tpl_path: &Path, vars: &HashMap<&str, String>) -> Result<String, String> {
    let content = fs::read_to_string(tpl_path)
        .map_err(|e| format!("Failed to read template {}: {}", tpl_path.display(), e))?;

    let mut result = content;
    for (key, val) in vars {
        let placeholder = format!("{{{{{}}}}}", key);
        result = result.replace(&placeholder, val);
    }
    Ok(result)
}

fn clean_directory(dir: &Path) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let _ = fs::remove_file(entry.path());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_template_and_site_detection() {
        let temp_dir = std::env::temp_dir().join(format!("bamp_cfg_test_{}", std::process::id()));
        let tpl_dir = temp_dir.join("templates/nginx");
        let www_dir = temp_dir.join("www/mysite/public");
        fs::create_dir_all(&tpl_dir).unwrap();
        fs::create_dir_all(&www_dir).unwrap();
        fs::write(www_dir.join("index.php"), "<?php echo 'hi';").unwrap();

        // Write a test template
        let vhost_tpl = tpl_dir.join("vhost.conf.template");
        fs::write(&vhost_tpl, "server { server_name {{domain}}; root {{document_root}}; }").unwrap();

        let manager = ConfigManager::new(temp_dir.clone());
        let sites = manager.refresh_sites().unwrap();

        assert_eq!(sites.len(), 1);
        assert_eq!(sites[0].domain, "mysite.test");
        assert!(sites[0].document_root.ends_with("public"));

        manager.render_all_configs().unwrap();

        let out_conf = temp_dir.join("data/nginx/vhosts/mysite.test.conf");
        assert!(out_conf.exists());
        let content = fs::read_to_string(out_conf).unwrap();
        assert!(content.contains("server_name mysite.test;"));
        assert!(content.contains(&sites[0].document_root));

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
