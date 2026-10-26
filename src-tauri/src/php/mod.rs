//! PHP Configuration & Extension Manager.
//! Parses and mutates `php.ini` for toggleable extensions and directives.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhpExtension {
    pub name: String,
    pub enabled: bool,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhpDirective {
    pub key: String,
    pub value: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhpConfig {
    pub ini_path: String,
    pub extensions: Vec<PhpExtension>,
    pub directives: Vec<PhpDirective>,
}

const COMMON_EXTENSIONS: &[(&str, &str)] = &[
    ("pdo_mysql", "MySQL PDO Database Driver"),
    ("mysqli", "MySQL Improved Extension"),
    ("xdebug", "Debugging and Profiling Tool"),
    ("intl", "Internationalization Functions"),
    ("gd", "Image Processing and GD Library"),
    ("zip", "Zip Archive Management"),
    ("redis", "Redis In-Memory Data Store Client"),
    ("opcache", "Opcode Cache Accelerator"),
    ("curl", "cURL Client Library"),
    ("mbstring", "Multibyte String Support"),
    ("fileinfo", "MIME Type Detection"),
    ("bcmath", "Arbitrary Precision Mathematics"),
    ("sodium", "Modern Cryptographic Library"),
    ("soap", "SOAP Protocol Client & Server"),
];

const COMMON_DIRECTIVES: &[(&str, &str, &str)] = &[
    ("memory_limit", "512M", "Maximum memory allocated per script"),
    ("upload_max_filesize", "128M", "Maximum allowed size for uploaded files"),
    ("post_max_size", "128M", "Maximum allowed size for POST request data"),
    ("max_execution_time", "300", "Maximum script execution time in seconds"),
    ("date.timezone", "UTC", "Default timezone used by date/time functions"),
    ("display_errors", "On", "Display PHP errors directly on screen"),
];

pub struct PhpManager;

impl PhpManager {
    /// Locate or initialize the primary `php.ini` file
    pub fn locate_php_ini(base_dir: &Path) -> PathBuf {
        let candidate_data = base_dir.join("data/php/php.ini");
        if candidate_data.exists() {
            return candidate_data;
        }

        // Check if there is an ini in bin/php/
        let candidate_bin = base_dir.join("bin/php/php.ini");
        if candidate_bin.exists() {
            return candidate_bin;
        }

        // Initialize default minimal php.ini in data/php/
        let php_dir = base_dir.join("data/php");
        let _ = fs::create_dir_all(&php_dir);
        let default_ini = candidate_data.clone();

        let mut content = String::from("; BAMP Default PHP Configuration\n[PHP]\n");
        for (key, val, _) in COMMON_DIRECTIVES {
            content.push_str(&format!("{} = {}\n", key, val));
        }
        content.push_str("\n; Extensions\n");
        for (ext, _) in COMMON_EXTENSIONS {
            let prefix = if *ext == "pdo_mysql" || *ext == "mysqli" || *ext == "mbstring" || *ext == "curl" {
                "extension="
            } else {
                ";extension="
            };
            content.push_str(&format!("{}{}\n", prefix, ext));
        }

        let _ = fs::write(&default_ini, content);
        default_ini
    }

    /// Read and parse PHP configuration (extensions and directives)
    pub fn get_php_config(base_dir: &Path) -> Result<PhpConfig, String> {
        let ini_path = Self::locate_php_ini(base_dir);
        let content = fs::read_to_string(&ini_path)
            .map_err(|e| format!("Failed to read php.ini at {}: {}", ini_path.display(), e))?;

        let mut extensions = Vec::new();
        for (ext, desc) in COMMON_EXTENSIONS {
            let ext_pattern = format!("extension={}", ext);
            let ext_pattern_dll = format!("extension={}.dll", ext);
            let ext_pattern_so = format!("extension={}.so", ext);
            let zend_pattern = format!("zend_extension={}", ext);

            let mut enabled = false;
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with(';') || trimmed.starts_with('#') {
                    continue;
                }
                if trimmed.contains(&ext_pattern)
                    || trimmed.contains(&ext_pattern_dll)
                    || trimmed.contains(&ext_pattern_so)
                    || trimmed.contains(&zend_pattern)
                {
                    enabled = true;
                    break;
                }
            }

            extensions.push(PhpExtension {
                name: ext.to_string(),
                enabled,
                description: desc.to_string(),
            });
        }

        let mut directives = Vec::new();
        for (key, default_val, desc) in COMMON_DIRECTIVES {
            let mut val = default_val.to_string();
            let key_prefix = format!("{} =", key);
            let key_prefix_alt = format!("{}=", key);

            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with(';') || trimmed.starts_with('#') {
                    continue;
                }
                if trimmed.starts_with(&key_prefix) || trimmed.starts_with(&key_prefix_alt) {
                    if let Some(pos) = trimmed.find('=') {
                        val = trimmed[pos + 1..].trim().trim_matches('"').trim_matches('\'').to_string();
                        break;
                    }
                }
            }

            directives.push(PhpDirective {
                key: key.to_string(),
                value: val,
                description: desc.to_string(),
            });
        }

        Ok(PhpConfig {
            ini_path: ini_path.to_string_lossy().to_string(),
            extensions,
            directives,
        })
    }

    /// Toggle a PHP extension enable/disable state in `php.ini`
    pub fn toggle_extension(base_dir: &Path, extension_name: &str, enable: bool) -> Result<bool, String> {
        let ini_path = Self::locate_php_ini(base_dir);
        let content = fs::read_to_string(&ini_path)
            .map_err(|e| format!("Failed to read php.ini: {}", e))?;

        let mut new_lines = Vec::new();
        let mut found = false;

        let ext_target = format!("extension={}", extension_name);
        let zend_target = format!("zend_extension={}", extension_name);

        for line in content.lines() {
            let trimmed = line.trim();
            let is_match = trimmed.contains(&ext_target) || trimmed.contains(&zend_target);

            if is_match {
                found = true;
                if enable {
                    // Strip leading semicolon/whitespace
                    let uncommented = trimmed.trim_start_matches(';').trim_start_matches('#').trim();
                    new_lines.push(uncommented.to_string());
                } else {
                    // Prepend semicolon if not present
                    if !trimmed.starts_with(';') {
                        new_lines.push(format!(";{}", trimmed));
                    } else {
                        new_lines.push(line.to_string());
                    }
                }
            } else {
                new_lines.push(line.to_string());
            }
        }

        // If not found in file and enabling, append it to the end
        if !found && enable {
            let entry = if extension_name == "xdebug" || extension_name == "opcache" {
                format!("zend_extension={}", extension_name)
            } else {
                format!("extension={}", extension_name)
            };
            new_lines.push(entry);
        }

        let new_content = new_lines.join("\n") + "\n";
        fs::write(&ini_path, new_content)
            .map_err(|e| format!("Failed to write updated php.ini: {}", e))?;

        Ok(enable)
    }

    /// Update a directive value in `php.ini`
    pub fn update_directive(base_dir: &Path, key: &str, value: &str) -> Result<String, String> {
        let ini_path = Self::locate_php_ini(base_dir);
        let content = fs::read_to_string(&ini_path)
            .map_err(|e| format!("Failed to read php.ini: {}", e))?;

        let mut new_lines = Vec::new();
        let mut found = false;
        let prefix1 = format!("{} =", key);
        let prefix2 = format!("{}=", key);

        for line in content.lines() {
            let trimmed = line.trim();
            let is_match = (trimmed.starts_with(&prefix1) || trimmed.starts_with(&prefix2))
                && !trimmed.starts_with(';')
                && !trimmed.starts_with('#');

            if is_match {
                found = true;
                new_lines.push(format!("{} = {}", key, value));
            } else {
                new_lines.push(line.to_string());
            }
        }

        if !found {
            new_lines.push(format!("{} = {}", key, value));
        }

        let new_content = new_lines.join("\n") + "\n";
        fs::write(&ini_path, new_content)
            .map_err(|e| format!("Failed to write updated php.ini: {}", e))?;

        Ok(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_php_ini_manipulation() {
        let temp_dir = std::env::temp_dir().join(format!("bamp_php_ini_test_{}", std::process::id()));
        let _ = fs::create_dir_all(&temp_dir);

        let config = PhpManager::get_php_config(&temp_dir).unwrap();
        assert!(!config.extensions.is_empty());

        // Toggle xdebug
        PhpManager::toggle_extension(&temp_dir, "xdebug", true).unwrap();
        let updated = PhpManager::get_php_config(&temp_dir).unwrap();
        let xdebug = updated.extensions.iter().find(|e| e.name == "xdebug").unwrap();
        assert!(xdebug.enabled);

        // Update memory_limit
        PhpManager::update_directive(&temp_dir, "memory_limit", "1024M").unwrap();
        let updated2 = PhpManager::get_php_config(&temp_dir).unwrap();
        let mem = updated2.directives.iter().find(|d| d.key == "memory_limit").unwrap();
        assert_eq!(mem.value, "1024M");

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
