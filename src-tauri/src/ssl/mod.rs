//! Local Root CA and SSL Certificate Generator.
//! Generates a local Root CA and issues wildcard `*.test` SSL certificates for HTTPS.

use rcgen::{
    BasicConstraints, CertificateParams, ExtendedKeyUsagePurpose, IsCa, KeyPair,
    KeyUsagePurpose,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SslPaths {
    pub ca_cert_path: String,
    pub ca_key_path: String,
    pub cert_path: String,
    pub key_path: String,
}

pub struct SslManager {
    data_dir: PathBuf,
}

impl SslManager {
    pub fn new(base_dir: &Path) -> Self {
        Self {
            data_dir: base_dir.join("data").join("ssl"),
        }
    }

    /// Ensure Root CA and wildcard certificates exist on disk.
    /// Returns paths to the generated certificates.
    pub fn ensure_certificates(&self) -> Result<SslPaths, String> {
        fs::create_dir_all(&self.data_dir)
            .map_err(|e| format!("Failed to create SSL data directory: {}", e))?;

        let ca_cert_path = self.data_dir.join("bamp-root-ca.crt");
        let ca_key_path = self.data_dir.join("bamp-root-ca.key");
        let cert_path = self.data_dir.join("cert.pem");
        let key_path = self.data_dir.join("key.pem");

        if ca_cert_path.exists() && ca_key_path.exists() && cert_path.exists() && key_path.exists()
        {
            return Ok(SslPaths {
                ca_cert_path: ca_cert_path.to_string_lossy().to_string(),
                ca_key_path: ca_key_path.to_string_lossy().to_string(),
                cert_path: cert_path.to_string_lossy().to_string(),
                key_path: key_path.to_string_lossy().to_string(),
            });
        }

        // 1. Generate Root CA
        let mut ca_params = CertificateParams::new(vec!["BAMP Local Root CA".to_string()])
            .map_err(|e| format!("CA params error: {}", e))?;
        ca_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        ca_params.distinguished_name.push(
            rcgen::DnType::OrganizationName,
            "BAMP Local Development",
        );
        ca_params.distinguished_name.push(
            rcgen::DnType::CommonName,
            "BAMP Local Root CA",
        );
        ca_params.key_usages.push(KeyUsagePurpose::KeyCertSign);
        ca_params.key_usages.push(KeyUsagePurpose::CrlSign);

        let ca_key = KeyPair::generate().map_err(|e| format!("CA keygen error: {}", e))?;
        let ca_cert = ca_params
            .self_signed(&ca_key)
            .map_err(|e| format!("CA self-sign error: {}", e))?;

        let ca_cert_pem = ca_cert.pem();
        let ca_key_pem = ca_key.serialize_pem();

        fs::write(&ca_cert_path, &ca_cert_pem)
            .map_err(|e| format!("Failed to write CA cert: {}", e))?;
        fs::write(&ca_key_path, &ca_key_pem)
            .map_err(|e| format!("Failed to write CA key: {}", e))?;

        // 2. Generate Wildcard End-Entity Certificate (*.test, localhost, 127.0.0.1)
        let subject_alt_names = vec![
            "*.test".to_string(),
            "test".to_string(),
            "localhost".to_string(),
            "127.0.0.1".to_string(),
        ];
        let mut leaf_params = CertificateParams::new(subject_alt_names)
            .map_err(|e| format!("Leaf params error: {}", e))?;
        leaf_params.distinguished_name.push(
            rcgen::DnType::CommonName,
            "*.test",
        );
        leaf_params.distinguished_name.push(
            rcgen::DnType::OrganizationName,
            "BAMP Local Development",
        );
        leaf_params.key_usages.push(KeyUsagePurpose::DigitalSignature);
        leaf_params
            .extended_key_usages
            .push(ExtendedKeyUsagePurpose::ServerAuth);
        leaf_params.use_authority_key_identifier_extension = true;

        let leaf_key = KeyPair::generate().map_err(|e| format!("Leaf keygen error: {}", e))?;
        let leaf_cert = leaf_params
            .signed_by(&leaf_key, &ca_cert, &ca_key)
            .map_err(|e| format!("Leaf signing error: {}", e))?;

        // Chain leaf + CA cert for Nginx/Apache fullchain bundle
        let fullchain_pem = format!("{}\n{}", leaf_cert.pem(), ca_cert_pem);
        let leaf_key_pem = leaf_key.serialize_pem();

        fs::write(&cert_path, &fullchain_pem)
            .map_err(|e| format!("Failed to write fullchain cert: {}", e))?;
        fs::write(&key_path, &leaf_key_pem)
            .map_err(|e| format!("Failed to write leaf key: {}", e))?;

        Ok(SslPaths {
            ca_cert_path: ca_cert_path.to_string_lossy().to_string(),
            ca_key_path: ca_key_path.to_string_lossy().to_string(),
            cert_path: cert_path.to_string_lossy().to_string(),
            key_path: key_path.to_string_lossy().to_string(),
        })
    }

    /// Install Root CA into system trust store.
    pub fn install_root_ca(&self) -> Result<String, String> {
        let ca_cert_path = self.data_dir.join("bamp-root-ca.crt");
        if !ca_cert_path.exists() {
            let _ = self.ensure_certificates()?;
        }

        #[cfg(target_os = "windows")]
        {
            let status = std::process::Command::new("certutil")
                .args(["-addstore", "-f", "Root", &ca_cert_path.to_string_lossy()])
                .status()
                .map_err(|e| format!("Failed to execute certutil: {}", e))?;

            if status.success() {
                Ok("BAMP Root CA installed into Windows Certificate Store successfully.".to_string())
            } else {
                Err("Failed to install Root CA via certutil. Elevation may be required.".to_string())
            }
        }

        #[cfg(target_os = "linux")]
        {
            // Try trust anchor (p11-kit / Arch / Fedora) first, then update-ca-certificates (Debian / Ubuntu)
            let cert_str = ca_cert_path.to_string_lossy().to_string();

            // Try pkexec trust anchor
            let status = std::process::Command::new("pkexec")
                .args(["trust", "anchor", "--store", &cert_str])
                .status();

            match status {
                Ok(s) if s.success() => return Ok("BAMP Root CA installed successfully via trust anchor.".to_string()),
                _ => {}
            }

            // Fallback: update-ca-certificates
            let install_script = format!(
                "cp '{}' /usr/local/share/ca-certificates/bamp-root-ca.crt && update-ca-certificates",
                cert_str
            );

            let status_debian = std::process::Command::new("pkexec")
                .args(["sh", "-c", &install_script])
                .status();

            match status_debian {
                Ok(s) if s.success() => Ok("BAMP Root CA installed via update-ca-certificates.".to_string()),
                _ => Err(format!(
                    "Could not automatically install Root CA with pkexec. Run manually:\n  sudo trust anchor --store '{}'\n  or\n  sudo cp '{}' /usr/local/share/ca-certificates/ && sudo update-ca-certificates",
                    cert_str, cert_str
                )),
            }
        }

        #[cfg(not(any(target_os = "windows", target_os = "linux")))]
        {
            Err("Automatic Root CA installation not supported on this platform.".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_verify_certificates() {
        let temp_dir = std::env::temp_dir().join(format!("bamp_ssl_test_{}", std::process::id()));
        let manager = SslManager::new(&temp_dir);

        let paths = manager.ensure_certificates().expect("Failed to generate certs");

        assert!(Path::new(&paths.ca_cert_path).exists());
        assert!(Path::new(&paths.ca_key_path).exists());
        assert!(Path::new(&paths.cert_path).exists());
        assert!(Path::new(&paths.key_path).exists());

        // Verify with openssl verify
        let status = std::process::Command::new("openssl")
            .args([
                "verify",
                "-CAfile",
                &paths.ca_cert_path,
                &paths.cert_path,
            ])
            .output();

        if let Ok(out) = status {
            let stdout = String::from_utf8_lossy(&out.stdout);
            assert!(stdout.contains("OK"), "openssl verify failed: {}", stdout);
        }

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
