use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use futures::future::join_all;
use tokio::net::UdpSocket;
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

use crate::cache::DnsCache;
use crate::matcher::Patterns;
use crate::routes::RouteManager;

const DNS_PORT: u16 = 53;
const MAX_PACKET_SIZE: usize = 4096;
const UPSTREAM_TIMEOUT: Duration = Duration::from_secs(5);

pub struct DnsServer {
    listen_addr: SocketAddr,
    non_matched_dns: Vec<IpAddr>,
    matched_dns: Vec<IpAddr>,
    patterns: Patterns,
    route_manager: Arc<RouteManager>,
    cache: Arc<DnsCache>,
}

impl DnsServer {
    pub fn new(
        listen_addr: SocketAddr,
        non_matched_dns: Vec<IpAddr>,
        matched_dns: Vec<IpAddr>,
        patterns: Patterns,
        route_manager: Arc<RouteManager>,
        cache: Arc<DnsCache>,
    ) -> Self {
        Self {
            listen_addr,
            non_matched_dns,
            matched_dns,
            patterns,
            route_manager,
            cache,
        }
    }

    pub async fn run(&self) -> Result<()> {
        let socket = UdpSocket::bind(self.listen_addr).await?;
        info!("DNS server listening on {}", self.listen_addr);

        let mut buf = [0u8; MAX_PACKET_SIZE];

        loop {
            let (len, client_addr) = socket.recv_from(&mut buf).await?;
            let start = Instant::now();
            let query = buf[..len].to_vec();

            let domain = match parse_domain_from_query(&query) {
                Some(d) => d,
                None => {
                    warn!("Failed to parse DNS query from {}", client_addr);
                    continue;
                }
            };

            let qtype = parse_qtype_from_query(&query).unwrap_or(0);
            let cache_key = format!("{}:{}", domain, qtype);

            // Check cache first
            if let Some((mut cached_response, cached_ips, was_matched)) = self.cache.get(&cache_key).await {
                // Copy transaction ID from query to cached response
                if cached_response.len() >= 2 && query.len() >= 2 {
                    cached_response[0] = query[0];
                    cached_response[1] = query[1];
                }
                // Refresh routes for matched entries
                if was_matched {
                    join_all(cached_ips.iter().map(|ip| self.route_manager.add_route(*ip))).await;
                }
                let total_time = start.elapsed();
                info!(
                    "{} (cached, {}) | total: {:.1}ms",
                    domain,
                    if was_matched { "matched" } else { "pass" },
                    total_time.as_secs_f64() * 1000.0
                );
                if let Err(e) = socket.send_to(&cached_response, client_addr).await {
                    error!("Failed to send cached response to {}: {}", client_addr, e);
                }
                continue;
            }

            let matched = self.patterns.matches(&domain);

            // Choose DNS servers based on match
            let servers = if matched && !self.matched_dns.is_empty() {
                &self.matched_dns
            } else {
                &self.non_matched_dns
            };

            let (response, upstream_server, upstream_time) = match self.forward_query(&query, servers).await {
                Ok(result) => result,
                Err(e) => {
                    error!("Failed to forward query for {}: {}", domain, e);
                    // Send SERVFAIL so client doesn't retry forever
                    if let Some(servfail) = build_servfail_response(&query) {
                        let _ = socket.send_to(&servfail, client_addr).await;
                    }
                    continue;
                }
            };

            let ips = parse_ips_from_response(&response);

            // Cache the response
            let ttl = parse_ttl_from_response(&response);
            self.cache.insert(&cache_key, response.clone(), ips.clone(), matched, ttl).await;

            if matched {
                join_all(ips.iter().map(|ip| self.route_manager.add_route(*ip))).await;
                let total_time = start.elapsed();
                info!(
                    "{} -> {:?} (matched, routed) | upstream: {} in {:.1}ms | total: {:.1}ms",
                    domain,
                    ips,
                    upstream_server,
                    upstream_time.as_secs_f64() * 1000.0,
                    total_time.as_secs_f64() * 1000.0
                );
            } else {
                let total_time = start.elapsed();
                debug!(
                    "{} | upstream: {} in {:.1}ms | total: {:.1}ms",
                    domain,
                    upstream_server,
                    upstream_time.as_secs_f64() * 1000.0,
                    total_time.as_secs_f64() * 1000.0
                );
            }

            if let Err(e) = socket.send_to(&response, client_addr).await {
                error!("Failed to send response to {}: {}", client_addr, e);
            }
        }
    }

    async fn forward_query(&self, query: &[u8], servers: &[IpAddr]) -> Result<(Vec<u8>, IpAddr, Duration)> {
        for server in servers {
            let addr = SocketAddr::new(*server, DNS_PORT);
            let start = Instant::now();
            match self.forward_to_server(query, addr).await {
                Ok(response) => {
                    let elapsed = start.elapsed();
                    return Ok((response, *server, elapsed));
                }
                Err(e) => {
                    debug!("Failed to query {}: {}", server, e);
                    continue;
                }
            }
        }

        anyhow::bail!("All upstream DNS servers failed")
    }

    async fn forward_to_server(&self, query: &[u8], addr: SocketAddr) -> Result<Vec<u8>> {
        let bind_addr: SocketAddr = if addr.is_ipv6() {
            SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), 0)
        } else {
            SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0)
        };
        let socket = UdpSocket::bind(bind_addr).await?;
        socket.send_to(query, addr).await?;

        let mut buf = [0u8; MAX_PACKET_SIZE];
        let (len, _) = timeout(UPSTREAM_TIMEOUT, socket.recv_from(&mut buf)).await??;

        Ok(buf[..len].to_vec())
    }
}

/// Parse query type from DNS query packet.
fn parse_qtype_from_query(packet: &[u8]) -> Option<u16> {
    if packet.len() < 12 {
        return None;
    }

    let mut pos = 12;

    // Skip the domain name
    while pos < packet.len() {
        let len = packet[pos] as usize;
        if len == 0 {
            pos += 1;
            break;
        }
        pos += 1 + len;
    }

    // Read QTYPE (2 bytes after domain)
    if pos + 2 <= packet.len() {
        Some(u16::from_be_bytes([packet[pos], packet[pos + 1]]))
    } else {
        None
    }
}

/// Skip a DNS name (domain) in a packet. Handles both uncompressed labels and compressed pointers.
/// Returns the new position after the name, or None if parsing fails.
fn skip_dns_name(packet: &[u8], mut pos: usize) -> Option<usize> {
    while pos < packet.len() {
        let b = packet[pos];
        if b == 0 {
            // Null terminator - end of uncompressed name
            return Some(pos + 1);
        } else if b & 0xC0 == 0xC0 {
            // Compression pointer (2 bytes) - end of name
            if pos + 2 > packet.len() {
                return None;
            }
            return Some(pos + 2);
        } else {
            // Label: length byte + label bytes
            let len = b as usize;
            pos += 1 + len;
            if pos > packet.len() {
                return None;
            }
        }
    }
    None // Ran off end of packet
}

/// Skip the entire question section (QDCOUNT questions).
/// Returns the new position after all questions, or None if parsing fails.
fn skip_question_section(packet: &[u8], mut pos: usize, qdcount: u16) -> Option<usize> {
    for _ in 0..qdcount {
        pos = skip_dns_name(packet, pos)?;
        pos += 4; // QTYPE (2) + QCLASS (2)
        if pos > packet.len() {
            return None;
        }
    }
    Some(pos)
}

/// Parse TTL from first answer in DNS response.
fn parse_ttl_from_response(packet: &[u8]) -> Option<u64> {
    if packet.len() < 12 {
        return None;
    }

    let ancount = u16::from_be_bytes([packet[6], packet[7]]);
    if ancount == 0 {
        return None;
    }

    let qdcount = u16::from_be_bytes([packet[4], packet[5]]);
    let mut pos = skip_question_section(packet, 12, qdcount)?;

    // Skip answer name (may be compressed)
    pos = skip_dns_name(packet, pos)?;

    // Read TTL (bytes 4-7 of the answer record, after TYPE and CLASS)
    if pos + 8 <= packet.len() {
        let ttl = u32::from_be_bytes([packet[pos + 4], packet[pos + 5], packet[pos + 6], packet[pos + 7]]);
        Some(ttl as u64)
    } else {
        None
    }
}

/// Parse domain name from DNS query packet.
fn parse_domain_from_query(packet: &[u8]) -> Option<String> {
    if packet.len() < 12 {
        return None;
    }

    let mut pos = 12;
    let mut domain_parts = Vec::new();

    while pos < packet.len() {
        let len = packet[pos] as usize;
        if len == 0 {
            break;
        }

        pos += 1;
        if pos + len > packet.len() {
            return None;
        }

        let part = std::str::from_utf8(&packet[pos..pos + len]).ok()?;
        domain_parts.push(part.to_string());
        pos += len;
    }

    if domain_parts.is_empty() {
        None
    } else {
        Some(domain_parts.join("."))
    }
}

/// Build a SERVFAIL response from a query (raw DNS packet manipulation).
fn build_servfail_response(query: &[u8]) -> Option<Vec<u8>> {
    if query.len() < 12 {
        return None;
    }
    let mut response = query.to_vec();
    // Byte 2: Set QR=1 (response), keep RD
    response[2] = 0x80 | (query[2] & 0x01);
    // Byte 3: Set RCODE=2 (SERVFAIL)
    response[3] = 0x02;
    Some(response)
}

/// Parse IP addresses from DNS response packet (A and AAAA records).
fn parse_ips_from_response(packet: &[u8]) -> Vec<IpAddr> {
    let mut ips = Vec::new();

    if packet.len() < 12 {
        return ips;
    }

    let ancount = u16::from_be_bytes([packet[6], packet[7]]) as usize;
    if ancount == 0 {
        return ips;
    }

    let qdcount = u16::from_be_bytes([packet[4], packet[5]]);
    let mut pos = match skip_question_section(packet, 12, qdcount) {
        Some(p) => p,
        None => return ips,
    };

    for _ in 0..ancount {
        // Skip answer name (may be compressed)
        pos = match skip_dns_name(packet, pos) {
            Some(p) => p,
            None => break,
        };

        if pos + 10 > packet.len() {
            break;
        }

        let rtype = u16::from_be_bytes([packet[pos], packet[pos + 1]]);
        let rdlength = u16::from_be_bytes([packet[pos + 8], packet[pos + 9]]) as usize;

        pos += 10;

        if pos + rdlength > packet.len() {
            break;
        }

        match rtype {
            1 if rdlength == 4 => {
                let ip = Ipv4Addr::new(
                    packet[pos],
                    packet[pos + 1],
                    packet[pos + 2],
                    packet[pos + 3],
                );
                ips.push(IpAddr::V4(ip));
            }
            28 if rdlength == 16 => {
                let mut octets = [0u8; 16];
                octets.copy_from_slice(&packet[pos..pos + 16]);
                let ip = Ipv6Addr::from(octets);
                ips.push(IpAddr::V6(ip));
            }
            _ => {}
        }

        pos += rdlength;
    }

    ips
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_domain() {
        let query = [
            0x00, 0x01, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07, 0x65,
            0x78, 0x61, 0x6d, 0x70, 0x6c, 0x65, 0x03, 0x63, 0x6f, 0x6d, 0x00, 0x00, 0x01, 0x00,
            0x01,
        ];
        let domain = parse_domain_from_query(&query);
        assert_eq!(domain, Some("example.com".to_string()));
    }

    #[test]
    fn test_parse_ips_with_compressed_qname() {
        // DNS response where question section uses a compressed QNAME pointer
        // This tests the fix for the bug where QTYPE's high byte (0x00) was
        // incorrectly treated as a null terminator after a compression pointer.
        #[rustfmt::skip]
        let response = [
            // Header (12 bytes)
            0x00, 0x01, // Transaction ID
            0x81, 0x80, // Flags: response, recursion desired/available
            0x00, 0x01, // QDCOUNT: 1
            0x00, 0x01, // ANCOUNT: 1
            0x00, 0x00, // NSCOUNT: 0
            0x00, 0x00, // ARCOUNT: 0
            // Question section: compressed QNAME pointer (e.g., pointing elsewhere)
            0xC0, 0x30, // Compression pointer (to offset 48, doesn't matter for this test)
            0x00, 0x01, // QTYPE: A (high byte 0x00 - this triggered the bug!)
            0x00, 0x01, // QCLASS: IN
            // Answer section
            0xC0, 0x0C, // Name: compression pointer to offset 12
            0x00, 0x01, // TYPE: A
            0x00, 0x01, // CLASS: IN
            0x00, 0x00, 0x01, 0x2C, // TTL: 300 seconds
            0x00, 0x04, // RDLENGTH: 4 bytes
            0x01, 0x02, 0x03, 0x04, // RDATA: IP 1.2.3.4
        ];
        let ips = parse_ips_from_response(&response);
        assert_eq!(ips, vec![IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4))]);
    }

    #[test]
    fn test_parse_ttl_with_compressed_qname() {
        // Same packet structure as above, testing TTL extraction
        #[rustfmt::skip]
        let response = [
            // Header (12 bytes)
            0x00, 0x01, // Transaction ID
            0x81, 0x80, // Flags
            0x00, 0x01, // QDCOUNT: 1
            0x00, 0x01, // ANCOUNT: 1
            0x00, 0x00, // NSCOUNT: 0
            0x00, 0x00, // ARCOUNT: 0
            // Question section: compressed QNAME pointer
            0xC0, 0x30, // Compression pointer
            0x00, 0x01, // QTYPE: A (high byte 0x00)
            0x00, 0x01, // QCLASS: IN
            // Answer section
            0xC0, 0x0C, // Name: compression pointer
            0x00, 0x01, // TYPE: A
            0x00, 0x01, // CLASS: IN
            0x00, 0x00, 0x01, 0x2C, // TTL: 300 seconds (0x12C)
            0x00, 0x04, // RDLENGTH: 4 bytes
            0x01, 0x02, 0x03, 0x04, // RDATA: IP 1.2.3.4
        ];
        let ttl = parse_ttl_from_response(&response);
        assert_eq!(ttl, Some(300));
    }

    #[test]
    fn test_parse_ips_uncompressed_qname() {
        // Standard DNS response with uncompressed QNAME (regression test)
        #[rustfmt::skip]
        let response = [
            // Header (12 bytes)
            0x00, 0x01, // Transaction ID
            0x81, 0x80, // Flags
            0x00, 0x01, // QDCOUNT: 1
            0x00, 0x01, // ANCOUNT: 1
            0x00, 0x00, // NSCOUNT: 0
            0x00, 0x00, // ARCOUNT: 0
            // Question section: uncompressed QNAME "example.com"
            0x07, b'e', b'x', b'a', b'm', b'p', b'l', b'e',
            0x03, b'c', b'o', b'm',
            0x00, // Null terminator
            0x00, 0x01, // QTYPE: A
            0x00, 0x01, // QCLASS: IN
            // Answer section
            0xC0, 0x0C, // Name: compression pointer to offset 12
            0x00, 0x01, // TYPE: A
            0x00, 0x01, // CLASS: IN
            0x00, 0x00, 0x00, 0x3C, // TTL: 60 seconds
            0x00, 0x04, // RDLENGTH: 4 bytes
            0x08, 0x08, 0x08, 0x08, // RDATA: IP 8.8.8.8
        ];
        let ips = parse_ips_from_response(&response);
        assert_eq!(ips, vec![IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))]);
    }

    #[test]
    fn test_skip_dns_name_uncompressed() {
        // "example.com" followed by QTYPE
        let packet = [
            0x07, b'e', b'x', b'a', b'm', b'p', b'l', b'e',
            0x03, b'c', b'o', b'm',
            0x00, // Null terminator
            0x00, 0x01, // QTYPE (should be at position 13)
        ];
        assert_eq!(skip_dns_name(&packet, 0), Some(13));
    }

    #[test]
    fn test_skip_dns_name_compressed() {
        // Compression pointer followed by QTYPE
        let packet = [
            0xC0, 0x20, // Compression pointer
            0x00, 0x01, // QTYPE (should be at position 2)
        ];
        assert_eq!(skip_dns_name(&packet, 0), Some(2));
    }
}
