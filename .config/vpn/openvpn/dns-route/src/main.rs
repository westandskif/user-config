mod cache;
mod dns;
mod matcher;
mod resolver;
mod route_store;
mod routes;
mod upstream;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use clap::Parser;
use tokio::signal;
use tokio::signal::unix::{signal as unix_signal, SignalKind};
use tracing::{error, info};

#[derive(Parser, Debug)]
#[command(name = "dns-route")]
#[command(about = "DNS server that routes matched domains through a specified gateway")]
struct Args {
    /// Comma-separated domain patterns (contains match)
    #[arg(long)]
    patterns: String,

    /// IPv4 gateway for matched traffic routes
    #[arg(long)]
    gateway_ipv4: String,

    /// IPv6 gateway for matched traffic routes (optional)
    #[arg(long)]
    gateway_ipv6: Option<String>,

    /// DNS servers for matched requests (comma-separated)
    #[arg(long)]
    dns_for_matched: String,

    /// DNS servers for non-matched requests (comma-separated)
    #[arg(long)]
    dns_for_non_matched: String,

    /// Listen address
    #[arg(long, default_value = "127.0.0.1:53")]
    listen: SocketAddr,

    /// Route expiration time in seconds
    #[arg(long, default_value = "300")]
    route_ttl: u64,

    /// Runtime directory for PID file, database, etc.
    #[arg(long, default_value = "/var/run/dns-route")]
    runtime_dir: PathBuf,
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let args = Args::parse();

    // Ensure runtime directory exists
    std::fs::create_dir_all(&args.runtime_dir)
        .with_context(|| format!("Failed to create runtime dir: {:?}", args.runtime_dir))?;

    let pid_file = args.runtime_dir.join("dns-route.pid");
    let db_path = args.runtime_dir.join("dns-route.db");

    let pid = std::process::id();
    std::fs::write(&pid_file, pid.to_string())
        .with_context(|| format!("Failed to write PID file: {:?}", pid_file))?;
    info!("Wrote PID {} to {:?}", pid, pid_file);

    let patterns = matcher::Patterns::new(&args.patterns);
    info!("Loaded {} patterns", patterns.len());

    let matched_dns_servers = resolver::parse_dns_servers(&args.dns_for_matched);
    let non_matched_dns_servers = resolver::parse_dns_servers(&args.dns_for_non_matched);

    if matched_dns_servers.is_empty() {
        anyhow::bail!("No DNS servers provided for matched requests");
    }
    if non_matched_dns_servers.is_empty() {
        anyhow::bail!("No DNS servers provided for non-matched requests");
    }

    info!("DNS for matched: {:?}", matched_dns_servers);
    info!("DNS for non-matched: {:?}", non_matched_dns_servers);
    info!("Route store database: {:?}", db_path);

    let route_manager = Arc::new(
        routes::RouteManager::new(
            args.gateway_ipv4.clone(),
            args.gateway_ipv6.clone(),
            args.route_ttl,
            db_path,
        )
        .await
        .with_context(|| "Failed to initialize route manager")?,
    );

    // Clean up orphan routes from previous crashed sessions
    route_manager.cleanup_orphans().await;

    let rm_cleanup = route_manager.clone();
    route_manager.start_cleanup_task();

    let dns_cache = Arc::new(cache::DnsCache::new(300));
    dns_cache.start_cleanup_task();

    let server = Arc::new(dns::DnsServer::new(
        args.listen,
        non_matched_dns_servers,
        matched_dns_servers,
        patterns,
        route_manager.clone(),
        dns_cache,
    ));

    info!("Starting DNS server on {}", args.listen);

    let mut sigterm = unix_signal(SignalKind::terminate())?;

    tokio::select! {
        result = server.run() => {
            if let Err(e) = result {
                error!("DNS server error: {}", e);
            }
        }
        _ = signal::ctrl_c() => {
            info!("Received Ctrl+C, shutting down");
        }
        _ = sigterm.recv() => {
            info!("Received SIGTERM, shutting down");
        }
    }

    info!("Cleaning up routes...");
    rm_cleanup.cleanup_all().await;

    let _ = std::fs::remove_file(&pid_file);

    info!("Shutdown complete");
    Ok(())
}
