use regex::Regex;
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::time::Instant;

/// Parsed DNS resolution: domain -> list of IPs
#[derive(Debug, Clone)]
pub struct DnsResolution {
    pub domain: String,
    pub ips: Vec<Ipv4Addr>,
}

/// Tracks DNS queries and correlates them with responses
pub struct DnsParser {
    /// Maps (client_ip, client_port, txn_id) -> (domain, timestamp)
    pending_queries: HashMap<(Ipv4Addr, u16, u16), (String, Instant)>,
    query_regex: Regex,
    response_regex: Regex,
    a_record_regex: Regex,
}

impl DnsParser {
    pub fn new() -> Self {
        Self {
            pending_queries: HashMap::new(),
            // Matches: "100.64.100.6.19767 > 100.64.100.1.53: 60595+ A? variations.brave.com. (38)"
            query_regex: Regex::new(
                r"(\d+\.\d+\.\d+\.\d+)\.(\d+) > \d+\.\d+\.\d+\.\d+\.53: (\d+)\+ A\? ([^\s]+)\. \("
            ).unwrap(),
            // Matches: "100.64.100.1.53 > 100.64.100.6.19767: 60595 5/0/0 ..."
            response_regex: Regex::new(
                r"\d+\.\d+\.\d+\.\d+\.53 > (\d+\.\d+\.\d+\.\d+)\.(\d+): (\d+) \d+/\d+/\d+"
            ).unwrap(),
            // Matches: "A 65.9.86.11"
            a_record_regex: Regex::new(r"A (\d+\.\d+\.\d+\.\d+)").unwrap(),
        }
    }

    /// Parse a tcpdump line, returns Some(DnsResolution) if this is a complete DNS response
    pub fn parse_line(&mut self, line: &str) -> Option<DnsResolution> {
        // Try to parse as query first
        if let Some(caps) = self.query_regex.captures(line) {
            let client_ip: Ipv4Addr = caps[1].parse().ok()?;
            let client_port: u16 = caps[2].parse().ok()?;
            let txn_id: u16 = caps[3].parse().ok()?;
            let domain = caps[4].to_string();

            self.pending_queries.insert((client_ip, client_port, txn_id), (domain, Instant::now()));
            return None;
        }

        // Try to parse as response
        if let Some(caps) = self.response_regex.captures(line) {
            let client_ip: Ipv4Addr = caps[1].parse().ok()?;
            let client_port: u16 = caps[2].parse().ok()?;
            let txn_id: u16 = caps[3].parse().ok()?;

            // Look up the domain from pending queries
            let (domain, _) = self.pending_queries.remove(&(client_ip, client_port, txn_id))?;

            // Extract all A records
            let ips: Vec<Ipv4Addr> = self.a_record_regex
                .captures_iter(line)
                .filter_map(|c| c[1].parse().ok())
                .collect();

            if ips.is_empty() {
                return None;
            }

            return Some(DnsResolution { domain, ips });
        }

        None
    }

    /// Clean up queries older than the given duration (to prevent memory leaks)
    pub fn cleanup_old_queries(&mut self, max_age_secs: u64) {
        let now = Instant::now();
        self.pending_queries.retain(|_, (_, timestamp)| {
            now.duration_since(*timestamp).as_secs() < max_age_secs
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_query_and_response() {
        let mut parser = DnsParser::new();

        // Query line
        let query = "22:03:21.802720 IP 100.64.100.6.19767 > 100.64.100.1.53: 60595+ A? variations.brave.com. (38)";
        assert!(parser.parse_line(query).is_none());

        // Response line
        let response = "22:03:21.837616 IP 100.64.100.1.53 > 100.64.100.6.19767: 60595 5/0/0 CNAME d17ndjuagurpsr.cloudfront.net., A 65.9.86.11, A 65.9.86.104, A 65.9.86.83, A 65.9.86.121 (145)";
        let result = parser.parse_line(response);

        assert!(result.is_some());
        let resolution = result.unwrap();
        assert_eq!(resolution.domain, "variations.brave.com");
        assert_eq!(resolution.ips.len(), 4);
    }

    #[test]
    fn test_different_clients_same_port_txnid() {
        let mut parser = DnsParser::new();

        // Query from client A
        let query_a = "22:03:21.802720 IP 192.168.1.10.12345 > 100.64.100.1.53: 1000+ A? example.com. (38)";
        parser.parse_line(query_a);

        // Query from client B with SAME port and txn_id
        let query_b = "22:03:21.802720 IP 192.168.1.20.12345 > 100.64.100.1.53: 1000+ A? other.com. (38)";
        parser.parse_line(query_b);

        // Response to client A - should get example.com, not other.com
        let response_a = "22:03:21.837616 IP 100.64.100.1.53 > 192.168.1.10.12345: 1000 1/0/0 A 1.1.1.1 (50)";
        let result_a = parser.parse_line(response_a).unwrap();
        assert_eq!(result_a.domain, "example.com");

        // Response to client B - should get other.com
        let response_b = "22:03:21.837616 IP 100.64.100.1.53 > 192.168.1.20.12345: 1000 1/0/0 A 2.2.2.2 (50)";
        let result_b = parser.parse_line(response_b).unwrap();
        assert_eq!(result_b.domain, "other.com");
    }

    #[test]
    fn test_slackb_com_many_a_records() {
        let mut parser = DnsParser::new();

        let query = "00:34:47.898188 IP 100.64.100.6.13359 > 100.64.100.1.53: 7495+ A? slackb.com. (28)";
        assert!(parser.parse_line(query).is_none());

        let response = "00:34:47.950266 IP 100.64.100.1.53 > 100.64.100.6.13359: 7495 10/0/0 A 44.216.98.239, A 52.3.167.79, A 3.90.158.208, A 44.205.171.153, A 34.195.221.192, A 52.200.46.145, A 54.236.104.103, A 18.213.32.120, A 3.91.140.69, A 34.202.253.140 (188)";
        let result = parser.parse_line(response).unwrap();

        assert_eq!(result.domain, "slackb.com");
        assert_eq!(result.ips.len(), 10);
        assert_eq!(result.ips[0], "44.216.98.239".parse::<Ipv4Addr>().unwrap());
        assert_eq!(result.ips[9], "34.202.253.140".parse::<Ipv4Addr>().unwrap());
    }

    #[test]
    fn test_youtube_with_cname() {
        let mut parser = DnsParser::new();

        let query = "00:35:31.758472 IP 100.64.100.6.38854 > 100.64.100.1.53: 22184+ A? www.youtube.com. (33)";
        assert!(parser.parse_line(query).is_none());

        let response = "00:35:31.805752 IP 100.64.100.1.53 > 100.64.100.6.38854: 22184 10/0/0 CNAME youtube-ui.l.google.com., A 142.250.179.142, A 172.217.168.238, A 142.250.179.174, A 142.250.179.206, A 142.251.142.206, A 142.251.39.142, A 172.217.168.206, A 172.217.23.206, A 216.58.208.110 (211)";
        let result = parser.parse_line(response).unwrap();

        assert_eq!(result.domain, "www.youtube.com");
        assert_eq!(result.ips.len(), 9);
        assert_eq!(result.ips[0], "142.250.179.142".parse::<Ipv4Addr>().unwrap());
        assert_eq!(result.ips[8], "216.58.208.110".parse::<Ipv4Addr>().unwrap());
    }
}
