use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use futures::future::join_all;
use tokio::net::UdpSocket;
use tokio::sync::{broadcast, Mutex};
use tracing::{debug, error, info, warn};

use crate::cache::DnsCache;
use crate::matcher::Patterns;
use crate::routes::RouteManager;
use crate::upstream::UpstreamSocketManager;

const DNS_PORT: u16 = 53;
const MAX_PACKET_SIZE: usize = 4096;
const UPSTREAM_TIMEOUT: Duration = Duration::from_secs(5);

/// Result broadcast to concurrent requests for the same domain
#[derive(Clone)]
struct PendingResult {
    response: Vec<u8>,
    ips: Vec<IpAddr>,
    matched: bool,
}

pub struct DnsServer {
    listen_addr: SocketAddr,
    non_matched_dns: Vec<IpAddr>,
    matched_dns: Vec<IpAddr>,
    patterns: Patterns,
    route_manager: Arc<RouteManager>,
    cache: Arc<DnsCache>,
    /// In-flight queries: cache_key -> broadcast channel for result
    pending_queries: Mutex<HashMap<String, broadcast::Sender<PendingResult>>>,
    /// Persistent sockets to upstream DNS servers
    upstream: Arc<UpstreamSocketManager>,
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
            pending_queries: Mutex::new(HashMap::new()),
            upstream: Arc::new(UpstreamSocketManager::new()),
        }
    }

    pub async fn run(self: Arc<Self>) -> Result<()> {
        let socket = Arc::new(UdpSocket::bind(self.listen_addr).await?);
        info!("DNS server listening on {}", self.listen_addr);

        loop {
            let mut buf = [0u8; MAX_PACKET_SIZE];
            let (len, client_addr) = socket.recv_from(&mut buf).await?;
            let query = buf[..len].to_vec();

            let socket = socket.clone();
            let server = self.clone();

            tokio::spawn(async move {
                if let Err(e) = server.handle_request(query, client_addr, &socket).await {
                    error!("Request handler error for {}: {}", client_addr, e);
                }
            });
        }
    }

    async fn handle_request(
        &self,
        query: Vec<u8>,
        client_addr: SocketAddr,
        socket: &UdpSocket,
    ) -> Result<()> {
        let start = Instant::now();

        let domain = match parse_domain_from_query(&query) {
            Some(d) => d,
            None => {
                warn!(
                    "Failed to parse domain from query ({}), forwarding as passthrough",
                    client_addr
                );
                return self.forward_passthrough(&query, client_addr, socket).await;
            }
        };

        let qtype = parse_qtype_from_query(&query).unwrap_or(0);
        let cache_key = format!("{}:{}", domain, qtype);

        // Check cache first
        if let Some((mut cached_response, cached_ips, was_matched)) =
            self.cache.get(&cache_key).await
        {
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
            return Ok(());
        }

        // Check if there's already an in-flight query for this domain
        {
            let pending = self.pending_queries.lock().await;
            if let Some(tx) = pending.get(&cache_key) {
                // Subscribe to existing in-flight query
                let mut rx = tx.subscribe();
                drop(pending); // Release lock while waiting

                match rx.recv().await {
                    Ok(result) => {
                        // Use result from the other request
                        let mut response = result.response;
                        // Copy transaction ID from our query
                        if response.len() >= 2 && query.len() >= 2 {
                            response[0] = query[0];
                            response[1] = query[1];
                        }
                        // Add routes if matched (before response)
                        if result.matched {
                            join_all(
                                result.ips.iter().map(|ip| self.route_manager.add_route(*ip)),
                            )
                            .await;
                        }
                        let total_time = start.elapsed();
                        info!(
                            "{} (coalesced, {}) | total: {:.1}ms",
                            domain,
                            if result.matched { "matched" } else { "pass" },
                            total_time.as_secs_f64() * 1000.0
                        );
                        socket.send_to(&response, client_addr).await?;
                        return Ok(());
                    }
                    Err(_) => {
                        // Sender dropped (query failed), fall through to do our own query
                    }
                }
            }
        }

        // We're the first (or previous query failed) - create broadcast channel and register
        let tx = {
            let mut pending = self.pending_queries.lock().await;
            let (tx, _) = broadcast::channel(16);
            pending.insert(cache_key.clone(), tx.clone());
            tx
        };

        // Do the actual upstream query (sequential server fallback preserved)
        let result = self.do_upstream_query(&query, &domain, &cache_key).await;

        // Remove from pending and broadcast result
        self.pending_queries.lock().await.remove(&cache_key);

        match result {
            Ok((response, ips, matched, upstream_server, upstream_time)) => {
                // Broadcast to waiters
                let _ = tx.send(PendingResult {
                    response: response.clone(),
                    ips: ips.clone(),
                    matched,
                });

                // Add routes if matched (before response)
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

                // Respond - copy transaction ID
                let mut final_response = response;
                if final_response.len() >= 2 && query.len() >= 2 {
                    final_response[0] = query[0];
                    final_response[1] = query[1];
                }
                socket.send_to(&final_response, client_addr).await?;
            }
            Err(e) => {
                error!("Failed to forward query for {}: {}", domain, e);
                // Send SERVFAIL so client doesn't retry forever
                if let Some(servfail) = build_servfail_response(&query) {
                    socket.send_to(&servfail, client_addr).await?;
                }
            }
        }

        Ok(())
    }

    /// Forward a query as pass-through when domain parsing fails.
    /// No caching, no routing - just relay the response from upstream.
    async fn forward_passthrough(
        &self,
        query: &[u8],
        client_addr: SocketAddr,
        socket: &UdpSocket,
    ) -> Result<()> {
        match self.forward_query(query, &self.non_matched_dns).await {
            Ok((mut response, server, _time)) => {
                // Copy transaction ID from query to response
                if response.len() >= 2 && query.len() >= 2 {
                    response[0] = query[0];
                    response[1] = query[1];
                }
                debug!("Passthrough query forwarded via {}", server);
                socket.send_to(&response, client_addr).await?;
            }
            Err(e) => {
                warn!("Passthrough forward failed: {}", e);
                if let Some(servfail) = build_servfail_response(query) {
                    socket.send_to(&servfail, client_addr).await?;
                }
            }
        }
        Ok(())
    }

    /// Perform upstream query, cache result, return (response, ips, matched, server, time)
    async fn do_upstream_query(
        &self,
        query: &[u8],
        domain: &str,
        cache_key: &str,
    ) -> Result<(Vec<u8>, Vec<IpAddr>, bool, IpAddr, Duration)> {
        let matched = self.patterns.matches(domain);
        let servers = if matched && !self.matched_dns.is_empty() {
            &self.matched_dns
        } else {
            &self.non_matched_dns
        };

        // forward_query still tries servers sequentially (unchanged)
        let (response, upstream_server, upstream_time) = self.forward_query(query, servers).await?;
        let ips = parse_ips_from_response(&response);

        // Conditional caching: skip truncated, error responses, and unparseable TTLs
        let rcode = parse_rcode_from_response(&response);
        let cacheable_rcode = matches!(rcode, Some(DnsRcode::NoError) | Some(DnsRcode::NxDomain));

        if is_truncated(&response) {
            debug!("Not caching response for {} (truncated)", domain);
        } else if !cacheable_rcode {
            debug!(
                "Not caching response for {} (rcode: {:?})",
                domain, rcode
            );
        } else if let Some(ttl_info) = parse_ttl_from_response(&response) {
            // Cap NXDOMAIN TTL to prevent long-lived negative cache entries
            let ttl = if matches!(rcode, Some(DnsRcode::NxDomain)) {
                ttl_info.ttl.min(300)
            } else {
                ttl_info.ttl
            };
            self.cache
                .insert(cache_key, response.clone(), ips.clone(), matched, Some(ttl))
                .await;
        } else {
            debug!("Not caching response for {} (TTL unparseable)", domain);
        }

        Ok((response, ips, matched, upstream_server, upstream_time))
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
        self.upstream.query(query, addr.ip(), UPSTREAM_TIMEOUT).await
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

/// Check if DNS response has TC (truncation) bit set.
/// TC bit indicates the response was truncated and client should retry via TCP.
fn is_truncated(packet: &[u8]) -> bool {
    // TC bit is bit 1 of byte 2 (flags field)
    packet.len() >= 3 && (packet[2] & 0x02) != 0
}

/// DNS response codes (RCODE)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DnsRcode {
    NoError,   // 0: No error
    ServFail,  // 2: Server failure
    NxDomain,  // 3: Non-existent domain
    Refused,   // 5: Query refused
    Other(u8), // Other codes
}

/// Parse RCODE from DNS response header.
/// RCODE is in the lower 4 bits of byte 3 (flags field).
fn parse_rcode_from_response(packet: &[u8]) -> Option<DnsRcode> {
    if packet.len() < 4 {
        return None;
    }
    let rcode = packet[3] & 0x0F;
    Some(match rcode {
        0 => DnsRcode::NoError,
        2 => DnsRcode::ServFail,
        3 => DnsRcode::NxDomain,
        5 => DnsRcode::Refused,
        n => DnsRcode::Other(n),
    })
}

/// TTL parsing result with context about source
#[derive(Debug)]
struct TtlInfo {
    ttl: u64,
    from_authority: bool,
}

/// Parse minimum TTL from DNS response.
/// For successful responses: returns minimum TTL across all answer records.
/// For NXDOMAIN: returns SOA TTL from authority section.
/// Returns None if no TTL can be determined.
fn parse_ttl_from_response(packet: &[u8]) -> Option<TtlInfo> {
    if packet.len() < 12 {
        return None;
    }

    let ancount = u16::from_be_bytes([packet[6], packet[7]]);
    let nscount = u16::from_be_bytes([packet[8], packet[9]]);
    let qdcount = u16::from_be_bytes([packet[4], packet[5]]);
    let mut pos = skip_question_section(packet, 12, qdcount)?;

    // First, try to get minimum TTL from answer section
    if ancount > 0 {
        let mut min_ttl: Option<u64> = None;

        for _ in 0..ancount {
            pos = skip_dns_name(packet, pos)?;
            if pos + 10 > packet.len() {
                break;
            }

            let ttl = u32::from_be_bytes([
                packet[pos + 4],
                packet[pos + 5],
                packet[pos + 6],
                packet[pos + 7],
            ]) as u64;

            min_ttl = Some(match min_ttl {
                Some(current) => current.min(ttl),
                None => ttl,
            });

            let rdlength = u16::from_be_bytes([packet[pos + 8], packet[pos + 9]]) as usize;
            pos += 10 + rdlength;

            if pos > packet.len() {
                break;
            }
        }

        return min_ttl.map(|ttl| TtlInfo {
            ttl,
            from_authority: false,
        });
    }

    // No answers - look for SOA in authority section (for NXDOMAIN)
    if nscount > 0 {
        for _ in 0..nscount {
            pos = skip_dns_name(packet, pos)?;
            if pos + 10 > packet.len() {
                break;
            }

            let rtype = u16::from_be_bytes([packet[pos], packet[pos + 1]]);
            let ttl = u32::from_be_bytes([
                packet[pos + 4],
                packet[pos + 5],
                packet[pos + 6],
                packet[pos + 7],
            ]) as u64;
            let rdlength = u16::from_be_bytes([packet[pos + 8], packet[pos + 9]]) as usize;

            // TYPE 6 = SOA
            if rtype == 6 {
                return Some(TtlInfo {
                    ttl,
                    from_authority: true,
                });
            }

            pos += 10 + rdlength;
            if pos > packet.len() {
                break;
            }
        }
    }

    None
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
        let ttl_info = parse_ttl_from_response(&response);
        assert!(ttl_info.is_some());
        assert_eq!(ttl_info.unwrap().ttl, 300);
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

    #[test]
    fn test_parse_rcode_noerror() {
        // Response with RCODE=0 (NoError)
        let response = [0x00, 0x01, 0x81, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        assert_eq!(parse_rcode_from_response(&response), Some(DnsRcode::NoError));
    }

    #[test]
    fn test_parse_rcode_servfail() {
        // Response with RCODE=2 (ServFail)
        let response = [0x00, 0x01, 0x81, 0x82, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        assert_eq!(parse_rcode_from_response(&response), Some(DnsRcode::ServFail));
    }

    #[test]
    fn test_parse_rcode_nxdomain() {
        // Response with RCODE=3 (NxDomain)
        let response = [0x00, 0x01, 0x81, 0x83, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        assert_eq!(parse_rcode_from_response(&response), Some(DnsRcode::NxDomain));
    }

    #[test]
    fn test_parse_rcode_refused() {
        // Response with RCODE=5 (Refused)
        let response = [0x00, 0x01, 0x81, 0x85, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        assert_eq!(parse_rcode_from_response(&response), Some(DnsRcode::Refused));
    }

    #[test]
    fn test_parse_min_ttl_multiple_answers() {
        // Response with 2 answers: TTL 300 and TTL 60, should return 60 (minimum)
        #[rustfmt::skip]
        let response = [
            // Header
            0x00, 0x01, // Transaction ID
            0x81, 0x80, // Flags
            0x00, 0x01, // QDCOUNT: 1
            0x00, 0x02, // ANCOUNT: 2
            0x00, 0x00, // NSCOUNT: 0
            0x00, 0x00, // ARCOUNT: 0
            // Question section: "example.com"
            0x07, b'e', b'x', b'a', b'm', b'p', b'l', b'e',
            0x03, b'c', b'o', b'm',
            0x00,       // Null terminator
            0x00, 0x01, // QTYPE: A
            0x00, 0x01, // QCLASS: IN
            // Answer 1
            0xC0, 0x0C, // Name: compression pointer
            0x00, 0x01, // TYPE: A
            0x00, 0x01, // CLASS: IN
            0x00, 0x00, 0x01, 0x2C, // TTL: 300 seconds
            0x00, 0x04, // RDLENGTH: 4 bytes
            0x01, 0x02, 0x03, 0x04, // RDATA: IP 1.2.3.4
            // Answer 2
            0xC0, 0x0C, // Name: compression pointer
            0x00, 0x01, // TYPE: A
            0x00, 0x01, // CLASS: IN
            0x00, 0x00, 0x00, 0x3C, // TTL: 60 seconds
            0x00, 0x04, // RDLENGTH: 4 bytes
            0x05, 0x06, 0x07, 0x08, // RDATA: IP 5.6.7.8
        ];
        let ttl_info = parse_ttl_from_response(&response);
        assert!(ttl_info.is_some());
        let info = ttl_info.unwrap();
        assert_eq!(info.ttl, 60);
        assert!(!info.from_authority);
    }

    #[test]
    fn test_parse_ttl_nxdomain_with_soa() {
        // NXDOMAIN response with SOA in authority section
        #[rustfmt::skip]
        let response = [
            // Header
            0x00, 0x01, // Transaction ID
            0x81, 0x83, // Flags: RCODE=3 (NXDOMAIN)
            0x00, 0x01, // QDCOUNT: 1
            0x00, 0x00, // ANCOUNT: 0
            0x00, 0x01, // NSCOUNT: 1 (SOA in authority)
            0x00, 0x00, // ARCOUNT: 0
            // Question section: "nonexistent.example.com"
            0x0B, b'n', b'o', b'n', b'e', b'x', b'i', b's', b't', b'e', b'n', b't',
            0x07, b'e', b'x', b'a', b'm', b'p', b'l', b'e',
            0x03, b'c', b'o', b'm',
            0x00,       // Null terminator
            0x00, 0x01, // QTYPE: A
            0x00, 0x01, // QCLASS: IN
            // Authority section: SOA record
            0x07, b'e', b'x', b'a', b'm', b'p', b'l', b'e',
            0x03, b'c', b'o', b'm',
            0x00,       // Null terminator
            0x00, 0x06, // TYPE: SOA (6)
            0x00, 0x01, // CLASS: IN
            0x00, 0x00, 0x00, 0x78, // TTL: 120 seconds
            0x00, 0x16, // RDLENGTH: 22 bytes (simplified SOA rdata)
            // Simplified SOA RDATA (just padding to match RDLENGTH)
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let ttl_info = parse_ttl_from_response(&response);
        assert!(ttl_info.is_some());
        let info = ttl_info.unwrap();
        assert_eq!(info.ttl, 120);
        assert!(info.from_authority);
    }

    #[test]
    fn test_is_truncated_true() {
        // Response with TC bit set (bit 1 of byte 2)
        let response = [0x00, 0x01, 0x82, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        assert!(is_truncated(&response));
    }

    #[test]
    fn test_is_truncated_false() {
        // Normal response without TC bit
        let response = [0x00, 0x01, 0x81, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        assert!(!is_truncated(&response));
    }

    #[test]
    fn test_is_truncated_short_packet() {
        // Packet too short to have flags
        let response = [0x00, 0x01];
        assert!(!is_truncated(&response));
    }
}
