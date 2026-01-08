use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::RwLock;
use tokio::time;
use tracing::debug;

struct CacheEntry {
    response: Vec<u8>,
    ips: Vec<IpAddr>,
    expires_at: Instant,
    matched: bool,
}

pub struct DnsCache {
    entries: RwLock<HashMap<String, CacheEntry>>,
    default_ttl: Duration,
}

impl DnsCache {
    pub fn new(default_ttl_secs: u64) -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            default_ttl: Duration::from_secs(default_ttl_secs),
        }
    }

    /// Get a cached response if it exists and hasn't expired.
    /// Returns (response, ips, matched).
    pub async fn get(&self, key: &str) -> Option<(Vec<u8>, Vec<IpAddr>, bool)> {
        let entries = self.entries.read().await;
        if let Some(entry) = entries.get(key) {
            if Instant::now() < entry.expires_at {
                debug!("Cache hit for {}", key);
                return Some((entry.response.clone(), entry.ips.clone(), entry.matched));
            }
        }
        None
    }

    /// Insert a response into the cache.
    pub async fn insert(
        &self,
        key: &str,
        response: Vec<u8>,
        ips: Vec<IpAddr>,
        matched: bool,
        ttl_secs: Option<u64>,
    ) {
        let ttl = ttl_secs.map(Duration::from_secs).unwrap_or(self.default_ttl);
        let entry = CacheEntry {
            response,
            ips,
            expires_at: Instant::now() + ttl,
            matched,
        };

        let mut entries = self.entries.write().await;
        entries.insert(key.to_string(), entry);
        debug!("Cached {} for {}s", key, ttl.as_secs());
    }

    /// Start the background cleanup task.
    pub fn start_cleanup_task(self: &Arc<Self>) {
        let cache = self.clone();
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                cache.cleanup_expired().await;
            }
        });
    }

    /// Remove expired entries (called periodically).
    async fn cleanup_expired(&self) {
        let now = Instant::now();
        let mut entries = self.entries.write().await;
        let before = entries.len();
        entries.retain(|_, entry| entry.expires_at > now);
        let removed = before - entries.len();
        if removed > 0 {
            debug!("Removed {} expired cache entries", removed);
        }
    }
}
