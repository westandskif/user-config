use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use futures::future::join_all;
use hickory_proto::op::Message;
use hickory_proto::rr::rdata::svcb::{SvcParamKey, SvcParamValue};
use hickory_proto::rr::RData;
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
const UPSTREAM_TIMEOUT_LAST: Duration = Duration::from_secs(10);

/// Result broadcast to concurrent requests for the same domain
#[derive(Clone)]
struct PendingResult {
    response: Vec<u8>,
    ips: Vec<IpAddr>,
    matched: bool,
}

/// Action to take for query coalescing
enum QueryAction {
    /// Wait on existing in-flight query
    Wait(broadcast::Receiver<PendingResult>),
    /// Execute the query ourselves (we're the owner)
    Execute(broadcast::Sender<PendingResult>),
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
                "{} {} (cached, {}) | total: {:.1}ms",
                domain,
                qtype_to_string(qtype),
                if was_matched { "matched" } else { "pass" },
                total_time.as_secs_f64() * 1000.0
            );
            if let Err(e) = socket.send_to(&cached_response, client_addr).await {
                error!("Failed to send cached response to {}: {}", client_addr, e);
            }
            return Ok(());
        }

        // Atomic check-and-insert for in-flight query coalescing
        loop {
            let action = {
                let mut pending = self.pending_queries.lock().await;
                match pending.entry(cache_key.clone()) {
                    Entry::Occupied(e) => QueryAction::Wait(e.get().subscribe()),
                    Entry::Vacant(e) => {
                        let (tx, _) = broadcast::channel(16);
                        e.insert(tx.clone());
                        QueryAction::Execute(tx)
                    }
                }
            };

            match action {
                QueryAction::Wait(mut rx) => {
                    match rx.recv().await {
                        Ok(result) => {
                            // Use result from the other request
                            let mut response = result.response;
                            if response.len() >= 2 && query.len() >= 2 {
                                response[0] = query[0];
                                response[1] = query[1];
                            }
                            if result.matched {
                                join_all(
                                    result.ips.iter().map(|ip| self.route_manager.add_route(*ip)),
                                )
                                .await;
                            }
                            let total_time = start.elapsed();
                            info!(
                                "{} {} (coalesced, {}) | total: {:.1}ms",
                                domain,
                                qtype_to_string(qtype),
                                if result.matched { "matched" } else { "pass" },
                                total_time.as_secs_f64() * 1000.0
                            );
                            socket.send_to(&response, client_addr).await?;
                            return Ok(());
                        }
                        Err(_) => {
                            // Owner dropped without sending - retry the loop
                            continue;
                        }
                    }
                }
                QueryAction::Execute(tx) => {
                    // We're the owner - do the upstream query
                    let result = self.do_upstream_query(&query, &domain, &cache_key).await;

                    match result {
                        Ok((response, ips, matched, upstream_server, upstream_time)) => {
                            // Broadcast to waiters FIRST (before removing from pending)
                            let _ = tx.send(PendingResult {
                                response: response.clone(),
                                ips: ips.clone(),
                                matched,
                            });

                            // Remove from pending AFTER broadcast to close the race window
                            self.pending_queries.lock().await.remove(&cache_key);

                            let total_time = start.elapsed();
                            if matched {
                                join_all(ips.iter().map(|ip| self.route_manager.add_route(*ip)))
                                    .await;
                                info!(
                                    "{} {} -> {} IPs (matched, routed) | upstream: {} in {:.1}ms | total: {:.1}ms",
                                    domain,
                                    qtype_to_string(qtype),
                                    ips.len(),
                                    upstream_server,
                                    upstream_time.as_secs_f64() * 1000.0,
                                    total_time.as_secs_f64() * 1000.0
                                );
                            } else {
                                info!(
                                    "{} {} | upstream: {} in {:.1}ms | total: {:.1}ms",
                                    domain,
                                    qtype_to_string(qtype),
                                    upstream_server,
                                    upstream_time.as_secs_f64() * 1000.0,
                                    total_time.as_secs_f64() * 1000.0
                                );
                            }

                            let mut final_response = response;
                            if final_response.len() >= 2 && query.len() >= 2 {
                                final_response[0] = query[0];
                                final_response[1] = query[1];
                            }
                            socket.send_to(&final_response, client_addr).await?;
                        }
                        Err(e) => {
                            // Remove from pending on error
                            self.pending_queries.lock().await.remove(&cache_key);

                            error!("Failed to forward query for {}: {}", domain, e);
                            if let Some(servfail) = build_servfail_response(&query) {
                                socket.send_to(&servfail, client_addr).await?;
                            }
                        }
                    }
                    return Ok(());
                }
            }
        }
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
        let last_idx = servers.len().saturating_sub(1);
        for (idx, server) in servers.iter().enumerate() {
            let is_last = idx == last_idx;
            let addr = SocketAddr::new(*server, DNS_PORT);
            let start = Instant::now();
            match self.forward_to_server(query, addr, is_last).await {
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

    async fn forward_to_server(&self, query: &[u8], addr: SocketAddr, is_last: bool) -> Result<Vec<u8>> {
        let timeout = if is_last { UPSTREAM_TIMEOUT_LAST } else { UPSTREAM_TIMEOUT };
        self.upstream.query(query, addr.ip(), timeout).await
    }
}

/// Parse query type from DNS query packet using hickory-proto.
fn parse_qtype_from_query(packet: &[u8]) -> Option<u16> {
    let message = Message::from_vec(packet).ok()?;
    Some(message.query()?.query_type().into())
}

/// Convert DNS query type (u16) to human-readable string.
fn qtype_to_string(qtype: u16) -> &'static str {
    match qtype {
        1 => "A",
        2 => "NS",
        5 => "CNAME",
        6 => "SOA",
        12 => "PTR",
        15 => "MX",
        16 => "TXT",
        28 => "AAAA",
        33 => "SRV",
        43 => "DS",
        46 => "RRSIG",
        47 => "NSEC",
        48 => "DNSKEY",
        52 => "TLSA",
        64 => "SVCB",
        65 => "HTTPS",
        255 => "ANY",
        _ => "?",
    }
}

/// Check if DNS response has TC (truncation) bit set using hickory-proto.
fn is_truncated(packet: &[u8]) -> bool {
    Message::from_vec(packet)
        .map(|m| m.truncated())
        .unwrap_or(false)
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

/// Parse RCODE from DNS response header using hickory-proto.
fn parse_rcode_from_response(packet: &[u8]) -> Option<DnsRcode> {
    use hickory_proto::op::ResponseCode;
    let message = Message::from_vec(packet).ok()?;
    let response_code = message.response_code();
    Some(match response_code {
        ResponseCode::NoError => DnsRcode::NoError,
        ResponseCode::ServFail => DnsRcode::ServFail,
        ResponseCode::NXDomain => DnsRcode::NxDomain,
        ResponseCode::Refused => DnsRcode::Refused,
        _ => DnsRcode::Other(u16::from(response_code) as u8),
    })
}

/// TTL parsing result with context about source
#[derive(Debug)]
struct TtlInfo {
    ttl: u64,
    from_authority: bool,
}

/// Parse minimum TTL from DNS response using hickory-proto.
/// For successful responses: returns minimum TTL across all answer records.
/// For NXDOMAIN: returns SOA TTL from authority section.
/// Returns None if no TTL can be determined.
fn parse_ttl_from_response(packet: &[u8]) -> Option<TtlInfo> {
    let message = Message::from_vec(packet).ok()?;

    // First, try to get minimum TTL from answer section
    if let Some(ttl) = message.answers().iter().map(|r| r.ttl() as u64).min() {
        return Some(TtlInfo {
            ttl,
            from_authority: false,
        });
    }

    // No answers - look for SOA in authority section (for NXDOMAIN)
    for record in message.name_servers() {
        if matches!(record.data(), RData::SOA(_)) {
            return Some(TtlInfo {
                ttl: record.ttl() as u64,
                from_authority: true,
            });
        }
    }

    None
}

/// Parse domain name from DNS query packet using hickory-proto.
fn parse_domain_from_query(packet: &[u8]) -> Option<String> {
    let message = Message::from_vec(packet).ok()?;
    let query = message.query()?;
    Some(query.name().to_string().trim_end_matches('.').to_string())
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

/// Parse IP addresses from DNS response packet (A, AAAA, and HTTPS/SVCB records).
/// For HTTPS/SVCB records, extracts ipv4hint and ipv6hint parameters.
/// Also parses A/AAAA records from the additional section (glue records).
fn parse_ips_from_response(packet: &[u8]) -> Vec<IpAddr> {
    let mut ips = Vec::new();

    let message = match Message::from_vec(packet) {
        Ok(m) => m,
        Err(_) => return ips,
    };

    // Parse answer section
    for record in message.answers() {
        match record.data() {
            RData::A(a) => {
                ips.push(IpAddr::V4(a.0));
            }
            RData::AAAA(aaaa) => {
                ips.push(IpAddr::V6(aaaa.0));
            }
            RData::HTTPS(https) => {
                extract_svcb_ips(https.0.svc_params(), &mut ips);
            }
            RData::SVCB(svcb) => {
                extract_svcb_ips(svcb.svc_params(), &mut ips);
            }
            _ => {}
        }
    }

    // Parse additional section (contains A/AAAA glue records for HTTPS/SVCB)
    for record in message.additionals() {
        match record.data() {
            RData::A(a) => {
                ips.push(IpAddr::V4(a.0));
            }
            RData::AAAA(aaaa) => {
                ips.push(IpAddr::V6(aaaa.0));
            }
            _ => {}
        }
    }

    ips
}

/// Extract IP addresses from SVCB/HTTPS ipv4hint and ipv6hint parameters.
fn extract_svcb_ips(params: &[(SvcParamKey, SvcParamValue)], ips: &mut Vec<IpAddr>) {
    for (key, value) in params {
        match (key, value) {
            (SvcParamKey::Ipv4Hint, SvcParamValue::Ipv4Hint(hint)) => {
                ips.extend(hint.0.iter().map(|a| IpAddr::V4(a.0)));
            }
            (SvcParamKey::Ipv6Hint, SvcParamValue::Ipv6Hint(hint)) => {
                ips.extend(hint.0.iter().map(|aaaa| IpAddr::V6(aaaa.0)));
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

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
