//! Embedded DNS Resolver for *.test local domains.
//! Listens on UDP 127.0.0.1:53 (or configured fallback port) and answers queries with 127.0.0.1.
//! Also provides hosts file synchronization fallback when port 53 cannot be bound.

use std::net::SocketAddr;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::net::UdpSocket;

/// Parses a DNS QNAME from wire format starting at offset into a dot-separated domain string.
pub fn parse_qname(buffer: &[u8], mut offset: usize) -> Result<(String, usize), String> {
    let mut labels = Vec::new();
    let mut jumped = false;
    let mut return_offset = 0;
    let max_jumps = 5;
    let mut jumps = 0;

    while offset < buffer.len() {
        let len = buffer[offset] as usize;
        if len == 0 {
            if !jumped {
                return_offset = offset + 1;
            }
            break;
        }

        // Check for pointer compression (0xC0 mask)
        if (len & 0xC0) == 0xC0 {
            if offset + 1 >= buffer.len() {
                return Err("Truncated pointer in DNS packet".to_string());
            }
            if !jumped {
                return_offset = offset + 2;
                jumped = true;
            }
            let pointer_offset = (((len & 0x3F) << 8) | (buffer[offset + 1] as usize)) as usize;
            offset = pointer_offset;
            jumps += 1;
            if jumps > max_jumps {
                return Err("Too many pointer jumps in DNS packet".to_string());
            }
            continue;
        }

        offset += 1;
        if offset + len > buffer.len() {
            return Err("DNS label length out of bounds".to_string());
        }

        let label = std::str::from_utf8(&buffer[offset..offset + len])
            .map_err(|e| format!("Invalid UTF-8 in DNS label: {}", e))?;
        labels.push(label.to_lowercase());
        offset += len;
    }

    let end_offset = if jumped { return_offset } else { offset + 1 };
    Ok((labels.join("."), end_offset))
}

/// Constructs a DNS response packet answering `127.0.0.1` for `.test` domains.
pub fn build_dns_response(query: &[u8], target_ip: [u8; 4]) -> Result<Vec<u8>, String> {
    if query.len() < 12 {
        return Err("DNS packet too short for header".to_string());
    }

    let tx_id = &query[0..2];
    let flags = &query[2..4];
    let qdcount = u16::from_be_bytes([query[4], query[5]]);
    if qdcount == 0 {
        return Err("No questions in DNS packet".to_string());
    }

    // Parse question section
    let (domain, offset) = parse_qname(query, 12)?;
    if offset + 4 > query.len() {
        return Err("Incomplete DNS question section".to_string());
    }

    let qtype = u16::from_be_bytes([query[offset], query[offset + 1]]);
    let qclass = u16::from_be_bytes([query[offset + 2], query[offset + 3]]);
    let question_end = offset + 4;

    let is_test_domain = domain.ends_with(".test") || domain == "test";
    let is_a_query = qtype == 1; // Type A

    let mut response = Vec::with_capacity(512);

    // Header: ID (2 bytes)
    response.extend_from_slice(tx_id);

    // Flags:
    // QR=1, AA=1, RD copied from query, RA=1
    let rd_bit = flags[0] & 0x01;
    let rcode = if is_test_domain && is_a_query { 0 } else { 3 }; // 0=NoError, 3=NXDomain
    let flag_byte1 = 0x84 | rd_bit; // QR(1) + AA(1) + RD
    let flag_byte2 = 0x80 | (rcode & 0x0F); // RA(1) + RCODE
    response.push(flag_byte1);
    response.push(flag_byte2);

    // Counts: QDCOUNT=1, ANCOUNT=(1 if matched else 0), NSCOUNT=0, ARCOUNT=0
    let ancount: u16 = if is_test_domain && is_a_query { 1 } else { 0 };
    response.extend_from_slice(&1u16.to_be_bytes());
    response.extend_from_slice(&ancount.to_be_bytes());
    response.extend_from_slice(&0u16.to_be_bytes());
    response.extend_from_slice(&0u16.to_be_bytes());

    // Echo Question Section
    response.extend_from_slice(&query[12..question_end]);

    // Answer Section (if matched)
    if is_test_domain && is_a_query {
        // Name compression pointer to offset 12 (0xC00C)
        response.extend_from_slice(&[0xC0, 0x0C]);
        // TYPE: A (1)
        response.extend_from_slice(&1u16.to_be_bytes());
        // CLASS: IN (1)
        response.extend_from_slice(&qclass.to_be_bytes());
        // TTL: 60 seconds
        response.extend_from_slice(&60u32.to_be_bytes());
        // RDLENGTH: 4 bytes (IPv4)
        response.extend_from_slice(&4u16.to_be_bytes());
        // RDATA: IPv4 address
        response.extend_from_slice(&target_ip);
    }

    Ok(response)
}

pub struct DnsServer {
    is_running: Arc<AtomicBool>,
}

impl DnsServer {
    pub fn new() -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Start listening on UDP port.
    /// If port 53 fails, logs clear diagnostic warning and returns Err.
    pub async fn start(&self, port: u16) -> Result<(), String> {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let socket = UdpSocket::bind(addr).await.map_err(|e| {
            format!(
                "Failed to bind DNS UDP socket on {}: {}. Will fallback to hosts file synchronization.",
                addr, e
            )
        })?;

        self.is_running.store(true, Ordering::SeqCst);
        let running = self.is_running.clone();

        tokio::spawn(async move {
            let mut buf = [0u8; 512];
            while running.load(Ordering::SeqCst) {
                match socket.recv_from(&mut buf).await {
                    Ok((amt, src)) => {
                        let query = &buf[..amt];
                        if let Ok(response) = build_dns_response(query, [127, 0, 0, 1]) {
                            let _ = socket.send_to(&response, src).await;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(())
    }

    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
    }
}

/// Fallback Hosts File Manager
pub struct HostsManager;

impl HostsManager {
    #[cfg(unix)]
    pub fn hosts_path() -> &'static Path {
        Path::new("/etc/hosts")
    }

    #[cfg(windows)]
    pub fn hosts_path() -> &'static Path {
        Path::new(r"C:\Windows\System32\drivers\etc\hosts")
    }

    /// Generate the BAMP hosts block containing all active domains
    pub fn format_hosts_block(domains: &[&str]) -> String {
        let mut block = String::from("\n# --- BAMP LOCAL DOMAINS BEGIN ---\n");
        for domain in domains {
            block.push_str(&format!("127.0.0.1  {}\n", domain));
        }
        block.push_str("# --- BAMP LOCAL DOMAINS END ---\n");
        block
    }

    /// Synchronize domains into the system hosts file
    pub fn sync_domains(domains: &[&str]) -> Result<(), String> {
        let path = Self::hosts_path();
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Could not read hosts file at {}: {}", path.display(), e))?;

        let block = Self::format_hosts_block(domains);

        // Strip previous BAMP block if present
        let mut new_content = String::new();
        let mut in_bamp_block = false;

        for line in content.lines() {
            if line.contains("# --- BAMP LOCAL DOMAINS BEGIN ---") {
                in_bamp_block = true;
                continue;
            }
            if line.contains("# --- BAMP LOCAL DOMAINS END ---") {
                in_bamp_block = false;
                continue;
            }
            if !in_bamp_block {
                new_content.push_str(line);
                new_content.push('\n');
            }
        }

        new_content.push_str(&block);

        std::fs::write(path, new_content)
            .map_err(|e| format!("Permission denied writing to {}: {}. Run with elevated privileges or execute manual sync.", path.display(), e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_parse_qname_and_build_response() {
        // Construct query for "myproject.test", Type A (1), Class IN (1)
        let mut packet = vec![
            0x12, 0x34, // ID
            0x01, 0x00, // Flags (standard query, recursion desired)
            0x00, 0x01, // QDCOUNT = 1
            0x00, 0x00, // ANCOUNT = 0
            0x00, 0x00, // NSCOUNT = 0
            0x00, 0x00, // ARCOUNT = 0
        ];

        // Question: 9myproject4test0
        packet.push(9);
        packet.extend_from_slice(b"myproject");
        packet.push(4);
        packet.extend_from_slice(b"test");
        packet.push(0);

        // QTYPE = 1 (A), QCLASS = 1 (IN)
        packet.extend_from_slice(&[0x00, 0x01, 0x00, 0x01]);

        let (domain, offset) = parse_qname(&packet, 12).unwrap();
        assert_eq!(domain, "myproject.test");
        assert_eq!(offset, 12 + 1 + 9 + 1 + 4 + 1);

        let response = build_dns_response(&packet, [127, 0, 0, 1]).unwrap();

        // Check ID
        assert_eq!(response[0], 0x12);
        assert_eq!(response[1], 0x34);

        // Check ANCOUNT == 1
        assert_eq!(response[6], 0x00);
        assert_eq!(response[7], 0x01);

        // Check IP 127.0.0.1 in last 4 bytes
        let len = response.len();
        assert_eq!(&response[len - 4..], &[127, 0, 0, 1]);
    }

    #[test]
    fn test_non_test_domain_returns_nxdomain() {
        let mut packet = vec![
            0xAB, 0xCD, // ID
            0x01, 0x00, // Flags
            0x00, 0x01, // QDCOUNT = 1
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        // 6google3com0
        packet.push(6);
        packet.extend_from_slice(b"google");
        packet.push(3);
        packet.extend_from_slice(b"com");
        packet.push(0);
        packet.extend_from_slice(&[0x00, 0x01, 0x00, 0x01]);

        let response = build_dns_response(&packet, [127, 0, 0, 1]).unwrap();

        // Check ANCOUNT == 0
        assert_eq!(response[6], 0x00);
        assert_eq!(response[7], 0x00);

        // RCODE in low 4 bits of byte 3 should be 3 (NXDomain)
        assert_eq!(response[3] & 0x0F, 3);
    }

    #[tokio::test]
    async fn test_dns_server_udp_socket_loopback() {
        let server = DnsServer::new();
        // Bind to ephemeral port for testing
        let socket = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
        let port = socket.local_addr().unwrap().port();
        drop(socket);

        let start_res = server.start(port).await;
        assert!(start_res.is_ok(), "DNS start failed: {:?}", start_res);

        // Send query using client UdpSocket
        let client = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        client.connect(SocketAddr::from(([127, 0, 0, 1], port))).await.unwrap();

        let mut query = vec![
            0x42, 0x42, // ID
            0x01, 0x00, // Flags
            0x00, 0x01, // QDCOUNT = 1
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        query.push(4);
        query.extend_from_slice(b"demo");
        query.push(4);
        query.extend_from_slice(b"test");
        query.push(0);
        query.extend_from_slice(&[0x00, 0x01, 0x00, 0x01]);

        client.send(&query).await.unwrap();

        let mut resp_buf = [0u8; 512];
        let amt = tokio::time::timeout(
            Duration::from_secs(2),
            client.recv(&mut resp_buf)
        ).await.expect("Timeout waiting for DNS response").unwrap();

        let resp = &resp_buf[..amt];
        assert_eq!(resp[0], 0x42);
        assert_eq!(resp[1], 0x42);
        assert_eq!(&resp[amt - 4..], &[127, 0, 0, 1]);

        server.stop();
    }
}
