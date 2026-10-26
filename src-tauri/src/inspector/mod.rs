//! Port Conflict Inspector.
//! Probes target service ports (e.g. 80, 443, 3306, 8025, 1025) and identifies
//! any conflicting processes and PIDs occupying those ports.

use serde::{Deserialize, Serialize};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PortConflict {
    pub port: u16,
    pub is_occupied: bool,
    pub pid: Option<u32>,
    pub process_name: Option<String>,
}

/// Check if a port is currently occupied on localhost.
pub fn is_port_in_use(port: u16) -> bool {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    // If we can connect to it, it is definitely in use
    if TcpStream::connect_timeout(&addr, Duration::from_millis(150)).is_ok() {
        return true;
    }
    // If bind fails, it's either in use or privileged
    match TcpListener::bind(addr) {
        Ok(_) => false,
        Err(_) => true,
    }
}

/// Inspect a port and attempt to find the conflicting process name and PID.
pub fn inspect_port(port: u16) -> PortConflict {
    let in_use = is_port_in_use(port);
    if !in_use {
        return PortConflict {
            port,
            is_occupied: false,
            pid: None,
            process_name: None,
        };
    }

    let (pid, process_name) = find_process_on_port(port);

    PortConflict {
        port,
        is_occupied: true,
        pid,
        process_name,
    }
}

/// Find owning PID and process name for a port.
#[cfg(target_os = "linux")]
fn find_process_on_port(port: u16) -> (Option<u32>, Option<String>) {
    // 1. Check /proc/net/tcp and tcp6 for the socket inode matching the port
    let inodes = get_socket_inodes_for_port(port);
    if !inodes.is_empty() {
        if let Some((pid, name)) = find_pid_by_socket_inodes(&inodes) {
            return (Some(pid), Some(name));
        }
    }

    // 2. Fallback: try `ss -tulpn` or `lsof` command parsing
    if let Ok(output) = std::process::Command::new("ss")
        .args(["-tulpn", &format!("sport = :{}", port)])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Some((pid, name)) = parse_ss_output(&stdout) {
            return (Some(pid), Some(name));
        }
    }

    (None, None)
}

#[cfg(target_os = "windows")]
fn find_process_on_port(port: u16) -> (Option<u32>, Option<String>) {
    // On Windows, use `netstat -ano -p tcp`
    if let Ok(output) = std::process::Command::new("netstat")
        .args(["-ano", "-p", "tcp"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 && parts[0] == "TCP" {
                if let Some(port_str) = parts[1].split(':').last() {
                    if port_str == port.to_string() {
                        if let Ok(pid) = parts[4].parse::<u32>() {
                            let name = get_process_name_by_pid(pid);
                            return (Some(pid), name);
                        }
                    }
                }
            }
        }
    }
    (None, None)
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn find_process_on_port(_port: u16) -> (Option<u32>, Option<String>) {
    (None, None)
}

#[cfg(target_os = "linux")]
fn get_socket_inodes_for_port(target_port: u16) -> Vec<u64> {
    let mut inodes = Vec::new();
    let port_hex = format!("{:04X}", target_port);

    for path in &["/proc/net/tcp", "/proc/net/tcp6"] {
        if let Ok(content) = std::fs::read_to_string(path) {
            for line in content.lines().skip(1) {
                let fields: Vec<&str> = line.split_whitespace().collect();
                if fields.len() > 9 {
                    let local_addr = fields[1];
                    if let Some(port_part) = local_addr.split(':').nth(1) {
                        if port_part.eq_ignore_ascii_case(&port_hex) {
                            if let Ok(inode) = fields[9].parse::<u64>() {
                                inodes.push(inode);
                            }
                        }
                    }
                }
            }
        }
    }
    inodes
}

#[cfg(target_os = "linux")]
fn find_pid_by_socket_inodes(target_inodes: &[u64]) -> Option<(u32, String)> {
    let proc_dir = std::fs::read_dir("/proc").ok()?;

    for entry in proc_dir.flatten() {
        let file_name = entry.file_name();
        let name_str = file_name.to_str()?;
        if let Ok(pid) = name_str.parse::<u32>() {
            let fd_dir = entry.path().join("fd");
            if let Ok(fd_entries) = std::fs::read_dir(fd_dir) {
                for fd_entry in fd_entries.flatten() {
                    if let Ok(link) = std::fs::read_link(fd_entry.path()) {
                        let link_str = link.to_string_lossy();
                        for inode in target_inodes {
                            let expected = format!("socket:[{}]", inode);
                            if link_str == expected {
                                let comm = std::fs::read_to_string(entry.path().join("comm"))
                                    .unwrap_or_else(|_| "unknown".to_string())
                                    .trim()
                                    .to_string();
                                return Some((pid, comm));
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

#[cfg(target_os = "linux")]
fn parse_ss_output(output: &str) -> Option<(u32, String)> {
    // Example: users:(("httpd",pid=769,fd=4))
    if let Some(users_idx) = output.find("users:((") {
        let rest = &output[users_idx + 8..];
        if let Some(quote_end) = rest.find('"') {
            let name = &rest[..quote_end];
            if let Some(pid_idx) = rest.find("pid=") {
                let pid_str = &rest[pid_idx + 4..];
                let pid_end = pid_str.find(|c: char| !c.is_ascii_digit()).unwrap_or(pid_str.len());
                if let Ok(pid) = pid_str[..pid_end].parse::<u32>() {
                    return Some((pid, name.to_string()));
                }
            }
        }
    }
    None
}

#[cfg(target_os = "windows")]
fn get_process_name_by_pid(pid: u32) -> Option<String> {
    if let Ok(output) = std::process::Command::new("tasklist")
        .args(["/FI", &format!("PID eq {}", pid), "/FO", "CSV", "/NH"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Some(first_line) = stdout.lines().next() {
            let parts: Vec<&str> = first_line.split(',').collect();
            if !parts.is_empty() {
                return Some(parts[0].trim_matches('"').to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_port_in_use_on_free_port() {
        // Ephemeral port should be free
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let free_port = listener.local_addr().unwrap().port();
        drop(listener); // close it

        assert!(!is_port_in_use(free_port));
    }

    #[test]
    fn test_is_port_in_use_on_bound_port() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        assert!(is_port_in_use(port));
    }
}
