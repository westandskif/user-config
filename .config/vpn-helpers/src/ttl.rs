use crate::db::{RouteDb, RouteEntry};
use crate::routes::delete_route;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{info, warn};

/// Clean up a list of routes from system and database
async fn cleanup_routes(db: &RouteDb, routes: Vec<RouteEntry>, log_reason: &str) {
    for entry in routes {
        // Delete from system routing table (also cleans up pf exception)
        if let Err(e) = delete_route(entry.ip, db).await {
            warn!(
                "Failed to delete system route for {}: {} (keeping DB entry for retry)",
                entry.ip, e
            );
            continue; // Skip DB deletion to allow retry on next cleanup cycle
        }

        // Delete from database
        if let Err(e) = db.delete_route(entry.ip).await {
            warn!("Failed to delete db entry for {}: {}", entry.ip, e);
        } else {
            info!(
                "Cleaned up {} route: {} (was for {})",
                log_reason, entry.ip, entry.domain
            );
        }
    }
}

/// Clean up expired routes (called on startup and periodically)
pub async fn cleanup_expired_routes(db: &RouteDb) {
    match db.get_expired_routes().await {
        Ok(expired) => {
            if !expired.is_empty() {
                info!("Found {} expired routes to clean up", expired.len());
                cleanup_routes(db, expired, "expired").await;
            }
        }
        Err(e) => {
            warn!("Failed to fetch expired routes: {}", e);
        }
    }
}

/// Clean up all routes (called on shutdown)
pub async fn cleanup_all_routes(db: &RouteDb) {
    match db.get_all_routes().await {
        Ok(routes) => {
            if !routes.is_empty() {
                info!("Cleaning up {} routes on shutdown", routes.len());
                cleanup_routes(db, routes, "shutdown").await;
            }
        }
        Err(e) => {
            warn!("Failed to fetch routes for cleanup: {}", e);
        }
    }
}

/// Clean up orphan pf exceptions (pf exceptions without corresponding routes)
/// This can happen if the process crashes after adding a pf exception but before adding the route
pub async fn cleanup_orphan_pf_exceptions(db: &RouteDb) {
    let pf_exceptions = match db.get_all_pf_exceptions().await {
        Ok(exceptions) => exceptions,
        Err(e) => {
            warn!("Failed to fetch pf exceptions: {}", e);
            return;
        }
    };

    let routes = match db.get_all_routes().await {
        Ok(r) => r,
        Err(e) => {
            warn!("Failed to fetch routes for orphan cleanup: {}", e);
            return;
        }
    };

    let route_ips: std::collections::HashSet<_> = routes.iter().map(|r| r.ip).collect();

    for ip in pf_exceptions {
        if !route_ips.contains(&ip) {
            info!("Found orphan pf exception: {} (no route entry)", ip);
            // Delete from system
            let _ = tokio::process::Command::new("sudo")
                .args([
                    "pfctl",
                    "-a",
                    "xvpn/killswitch",
                    "-t",
                    "xvpn_killswitch_exceptions",
                    "-T",
                    "delete",
                    &ip.to_string(),
                ])
                .output()
                .await;
            // Delete from DB
            if let Err(e) = db.delete_pf_exception(ip).await {
                warn!("Failed to delete orphan pf exception from DB for {}: {}", ip, e);
            } else {
                info!("Cleaned up orphan pf exception: {}", ip);
            }
        }
    }
}

/// Spawn a background task that periodically cleans up expired routes
pub fn spawn_cleanup_task(db: Arc<RouteDb>, check_interval_secs: u64) {
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(check_interval_secs));

        loop {
            ticker.tick().await;
            cleanup_expired_routes(&db).await;
        }
    });
}
