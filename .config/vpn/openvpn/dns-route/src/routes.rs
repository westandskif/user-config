use std::collections::HashMap;
use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use tokio::process::Command;
use tokio::sync::RwLock;

use anyhow::Result;
use tokio::time;
use tracing::{debug, error, info, warn};

use crate::route_store::RouteStore;

/// In-memory tracker to avoid unnecessary route refresh operations under high QPS.
/// Only refreshes routes when they're within the threshold of TTL remaining.
struct RouteRefresher {
    last_refresh: RwLock<HashMap<IpAddr, Instant>>,
    ttl_secs: u64,
    /// Refresh when less than this fraction of TTL remains (0.25 = 25%)
    refresh_threshold: f64,
}

impl RouteRefresher {
    fn new(ttl_secs: u64) -> Self {
        Self {
            last_refresh: RwLock::new(HashMap::new()),
            ttl_secs,
            refresh_threshold: 0.25,
        }
    }

    /// Returns true if the route should be refreshed (never seen or near expiry).
    async fn should_refresh(&self, ip: IpAddr) -> bool {
        let entries = self.last_refresh.read().await;
        match entries.get(&ip) {
            None => true,
            Some(last) => {
                let elapsed = last.elapsed().as_secs();
                let refresh_after = ((1.0 - self.refresh_threshold) * self.ttl_secs as f64) as u64;
                elapsed >= refresh_after
            }
        }
    }

    async fn mark_refreshed(&self, ip: IpAddr) {
        self.last_refresh.write().await.insert(ip, Instant::now());
    }

    /// Remove entries older than TTL (no longer relevant).
    async fn cleanup_stale(&self) {
        let ttl = Duration::from_secs(self.ttl_secs);
        self.last_refresh
            .write()
            .await
            .retain(|_, last| last.elapsed() < ttl);
    }
}

/// Manages routes with expiration.
pub struct RouteManager {
    gateway_ipv4: String,
    gateway_ipv6: Option<String>,
    ttl_secs: u64,
    store: RouteStore,
    run_id: String,
    refresher: RouteRefresher,
}

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

impl RouteManager {
    pub async fn new(
        gateway_ipv4: String,
        gateway_ipv6: Option<String>,
        ttl_secs: u64,
        db_path: PathBuf,
    ) -> Result<Self> {
        let run_id = format!(
            "{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        );

        let store = RouteStore::new(db_path).await?;
        store.init(&run_id, std::process::id()).await?;

        Ok(Self {
            gateway_ipv4,
            gateway_ipv6,
            ttl_secs,
            store,
            run_id,
            refresher: RouteRefresher::new(ttl_secs),
        })
    }

    /// Add a route for the given IP address through the gateway.
    /// If the route already exists and was recently refreshed, skip.
    /// If the route exists but is near expiry, refresh its expiration time.
    pub async fn add_route(&self, ip: IpAddr) {
        // Fast path: skip if recently refreshed (avoids DB operations under high QPS)
        if !self.refresher.should_refresh(ip).await {
            return;
        }

        // Get appropriate gateway for IP version
        let gateway = if ip.is_ipv6() {
            match &self.gateway_ipv6 {
                Some(gw) => gw,
                None => {
                    debug!("Skipping IPv6 route for {} (no IPv6 gateway)", ip);
                    return;
                }
            }
        } else {
            &self.gateway_ipv4
        };

        let ip_str = ip.to_string();
        let expires_at = now_unix() + self.ttl_secs as i64;
        let family: i64 = if ip.is_ipv6() { 6 } else { 4 };

        // Check if route already exists
        let exists = match self.store.route_exists(&ip_str).await {
            Ok(exists) => exists,
            Err(e) => {
                warn!("Failed to check route existence: {}", e);
                false
            }
        };

        if exists {
            // Just refresh expiration
            if let Err(e) = self.store.upsert_route(&ip_str, family, gateway, expires_at, &self.run_id).await {
                warn!("Failed to refresh route in store: {}", e);
            } else {
                self.refresher.mark_refreshed(ip).await;
            }
            debug!("Refreshed route for {}", ip);
            return;
        }

        // Create new route
        if self.create_system_route(ip, gateway).await {
            if let Err(e) = self.store.upsert_route(&ip_str, family, gateway, expires_at, &self.run_id).await {
                warn!("Failed to persist route to store: {}", e);
            } else {
                self.refresher.mark_refreshed(ip).await;
            }
            info!("Added route for {} via {}", ip, gateway);
        }
    }

    /// Start the background cleanup task.
    pub fn start_cleanup_task(self: &Arc<Self>) {
        let manager = self.clone();
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                manager.cleanup_expired().await;
                manager.refresher.cleanup_stale().await;
            }
        });
    }

    /// Remove expired routes.
    async fn cleanup_expired(&self) {
        let now = now_unix();
        match self.store.list_expired_routes(now, &self.run_id).await {
            Ok(expired) => {
                for route in expired {
                    if let Ok(ip) = route.ip.parse::<IpAddr>() {
                        if self.delete_system_route(ip).await {
                            if let Err(e) = self.store.delete_route(&route.ip).await {
                                warn!("Failed to delete route from store: {}", e);
                            }
                            info!("Removed expired route for {}", ip);
                        }
                    }
                }
            }
            Err(e) => {
                warn!("Failed to list expired routes from store: {}", e);
            }
        }
    }

    /// Clean up all managed routes (called on shutdown).
    pub async fn cleanup_all(&self) {
        match self.store.list_run_routes(&self.run_id).await {
            Ok(routes) => {
                for route in routes {
                    if let Ok(ip) = route.ip.parse::<IpAddr>() {
                        if self.delete_system_route(ip).await {
                            // Only delete from DB on success
                            if let Err(e) = self.store.delete_route(&route.ip).await {
                                warn!("Failed to delete route from store: {}", e);
                            }
                            info!("Cleaned up route for {}", ip);
                        } else {
                            warn!("Failed to delete OS route for {}, keeping in DB for retry", ip);
                        }
                    }
                }
                // Always mark run as ended - failed routes stay in DB for cleanup_orphans
                if let Err(e) = self.store.record_run_end(&self.run_id).await {
                    warn!("Failed to record run end: {}", e);
                }
            }
            Err(e) => {
                warn!("Failed to list routes from store: {}", e);
            }
        }
    }

    /// Clean up orphan routes from previous crashed sessions.
    pub async fn cleanup_orphans(&self) {
        match self.store.list_orphan_routes(&self.run_id).await {
            Ok(orphans) => {
                if orphans.is_empty() {
                    return;
                }
                info!("Found {} orphan routes from previous sessions", orphans.len());
                for route in orphans {
                    if let Ok(ip) = route.ip.parse::<IpAddr>() {
                        if self.delete_system_route(ip).await {
                            // Only delete from DB on success
                            if let Err(e) = self.store.delete_route(&route.ip).await {
                                warn!("Failed to delete orphan route from store: {}", e);
                            }
                            info!("Cleaned up orphan route for {}", ip);
                        } else {
                            warn!("Failed to delete orphan OS route for {}, will retry next startup", ip);
                        }
                    } else {
                        // Invalid IP in DB - delete the record
                        if let Err(e) = self.store.delete_route(&route.ip).await {
                            warn!("Failed to delete invalid orphan route from store: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                warn!("Failed to list orphan routes: {}", e);
            }
        }
    }

    /// Create a system route using /sbin/route.
    async fn create_system_route(&self, ip: IpAddr, gateway: &str) -> bool {
        let mut cmd = Command::new("/sbin/route");
        cmd.arg("add");
        if ip.is_ipv6() {
            cmd.arg("-inet6");
        }
        cmd.arg("-host").arg(ip.to_string()).arg(gateway);
        let result = cmd.output().await;

        match result {
            Ok(output) => {
                if output.status.success() {
                    true
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    if stderr.contains("File exists") {
                        debug!("Route for {} already exists", ip);
                        true
                    } else {
                        warn!("Failed to add route for {}: {}", ip, stderr.trim());
                        false
                    }
                }
            }
            Err(e) => {
                error!("Failed to execute route command: {}", e);
                false
            }
        }
    }

    /// Delete a system route using /sbin/route.
    async fn delete_system_route(&self, ip: IpAddr) -> bool {
        let mut cmd = Command::new("/sbin/route");
        cmd.arg("delete");
        if ip.is_ipv6() {
            cmd.arg("-inet6");
        }
        cmd.arg("-host").arg(ip.to_string());
        let result = cmd.output().await;

        match result {
            Ok(output) => {
                if output.status.success() {
                    true
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    if stderr.contains("not in table") {
                        debug!("Route for {} not in table", ip);
                        true
                    } else {
                        warn!("Failed to delete route for {}: {}", ip, stderr.trim());
                        false
                    }
                }
            }
            Err(e) => {
                error!("Failed to execute route command: {}", e);
                false
            }
        }
    }
}
