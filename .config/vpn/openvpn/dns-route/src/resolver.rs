use std::net::IpAddr;

use tracing::warn;

/// Parse DNS servers from a comma-separated string.
pub fn parse_dns_servers(servers_str: &str) -> Vec<IpAddr> {
    servers_str
        .split(',')
        .filter_map(|s| {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                return None;
            }
            match trimmed.parse::<IpAddr>() {
                Ok(ip) => Some(ip),
                Err(_) => {
                    warn!("Invalid DNS server IP: {}", trimmed);
                    None
                }
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_dns_servers() {
        let servers = parse_dns_servers("8.8.8.8, 1.1.1.1, invalid");
        assert_eq!(servers.len(), 2);
    }

    #[test]
    fn test_parse_empty() {
        let servers = parse_dns_servers("");
        assert_eq!(servers.len(), 0);
    }
}
