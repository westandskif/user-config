use crate::db::RouteDb;
use std::net::Ipv4Addr;
use tokio::process::Command;
use tracing::{info, warn};

/// Add a route for the given IP via the specified gateway (macOS)
pub async fn add_route(ip: Ipv4Addr, gateway: Ipv4Addr, db: &RouteDb) -> Result<(), String> {
    // Record pf exception in DB first (for crash recovery)
    if let Err(e) = db.add_pf_exception(ip).await {
        warn!("Failed to record pf exception in DB for {}: {}", ip, e);
    }

    // Add pf exception to ensure firewall allows traffic before route is added
    let pf_output = Command::new("sudo")
        .args([
            "pfctl",
            "-a",
            "xvpn/killswitch",
            "-t",
            "xvpn_killswitch_exceptions",
            "-T",
            "add",
            &ip.to_string(),
        ])
        .output()
        .await
        .map_err(|e| format!("Failed to execute pfctl command: {}", e))?;

    if pf_output.status.success() {
        info!("Added pf exception: {}", ip);
    } else {
        let stderr = String::from_utf8_lossy(&pf_output.stderr);
        // "already exists" is not a real error
        if !stderr.contains("already exists") && !pf_output.status.success() {
            // Rollback DB entry
            let _ = db.delete_pf_exception(ip).await;
            warn!("Failed to add pf exception for {}: {}", ip, stderr);
            return Err(format!("pf exception add failed: {}", stderr));
        }
        info!("pf exception already exists: {}", ip);
    }

    let output = Command::new("route")
        .args(["-n", "add", "-host", &ip.to_string(), &gateway.to_string()])
        .output()
        .await
        .map_err(|e| format!("Failed to execute route command: {}", e))?;

    if output.status.success() {
        info!("Added route: {} via {}", ip, gateway);
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // "File exists" means route already exists, not a real error
        if stderr.contains("File exists") {
            info!("Route already exists: {} via {}", ip, gateway);
            Ok(())
        } else {
            // Rollback: remove pf exception from system and DB
            let _ = Command::new("sudo")
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
            let _ = db.delete_pf_exception(ip).await;
            warn!(
                "Failed to add route {} via {}: {} (rolled back pf exception)",
                ip, gateway, stderr
            );
            Err(format!("Route add failed: {}", stderr))
        }
    }
}

/// Delete a route for the given IP (macOS)
pub async fn delete_route(ip: Ipv4Addr, db: &RouteDb) -> Result<(), String> {
    let output = Command::new("route")
        .args(["-n", "delete", "-host", &ip.to_string()])
        .output()
        .await
        .map_err(|e| format!("Failed to execute route command: {}", e))?;

    let route_ok = if output.status.success() {
        info!("Deleted route: {}", ip);
        true
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // "not in table" means route doesn't exist, not a real error
        if stderr.contains("not in table") {
            info!("Route already gone: {}", ip);
            true
        } else {
            warn!("Failed to delete route {}: {}", ip, stderr);
            return Err(format!("Route delete failed: {}", stderr));
        }
    };

    // Delete pf exception after route is removed
    if route_ok {
        let pf_output = Command::new("sudo")
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
            .await
            .map_err(|e| format!("Failed to execute pfctl command: {}", e))?;

        if pf_output.status.success() {
            info!("Deleted pf exception: {}", ip);
        } else {
            let stderr = String::from_utf8_lossy(&pf_output.stderr);
            // Entry not in table is not a real error
            if !stderr.contains("not in table") {
                warn!("Failed to delete pf exception for {}: {}", ip, stderr);
            } else {
                info!("pf exception already gone: {}", ip);
            }
        }

        // Remove from DB tracking
        if let Err(e) = db.delete_pf_exception(ip).await {
            warn!("Failed to delete pf exception from DB for {}: {}", ip, e);
        }
    }

    Ok(())
}
