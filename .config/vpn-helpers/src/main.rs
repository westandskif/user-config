mod db;
mod matcher;
mod parser;
mod routes;
mod ttl;

use clap::Parser;
use db::RouteDb;
use futures::future::join_all;
use matcher::matches_patterns;
use parser::DnsParser;
use routes::add_route;
use std::net::Ipv4Addr;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing::{info, warn, Level};

#[derive(Parser, Debug)]
#[command(name = "dns-route-manager")]
#[command(about = "Opportunistic route management based on DNS queries")]
struct Args {
    /// Gateway IP for matched routes
    #[arg(short, long)]
    gateway: Ipv4Addr,

    /// Comma-separated domain suffix patterns (e.g., ".by,.ru")
    #[arg(short, long, value_delimiter = ',')]
    patterns: Vec<String>,

    /// Route TTL in seconds (default: 300 = 5 minutes)
    #[arg(short, long, default_value = "300")]
    ttl: u64,

    /// SQLite database path
    #[arg(short, long, default_value = "./routes.db")]
    db: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();

    let args = Args::parse();

    info!("Starting dns-route-manager");
    info!("Gateway: {}", args.gateway);
    info!("Patterns: {:?}", args.patterns);
    info!("TTL: {} seconds", args.ttl);

    // Initialize database
    let db = Arc::new(RouteDb::new(&args.db).await?);
    info!("Database initialized at {}", args.db);

    // Clean up any stale routes and orphan pf exceptions from previous runs
    info!("Checking for stale routes from previous run...");
    ttl::cleanup_expired_routes(&db).await;
    ttl::cleanup_orphan_pf_exceptions(&db).await;

    // Spawn TTL cleanup task (runs every 30 seconds)
    ttl::spawn_cleanup_task(Arc::clone(&db), 30);
    info!("TTL cleanup task started (checking every 30s)");

    // Run the main processing loop with signal handling
    let result = run_main_loop(&db, &args).await;

    // Graceful shutdown: clean up all routes
    info!("Shutting down, cleaning up routes...");
    ttl::cleanup_all_routes(&db).await;
    info!("Shutdown complete");

    result
}

async fn run_main_loop(db: &Arc<RouteDb>, args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize parser
    let mut dns_parser = DnsParser::new();
    let mut cleanup_counter = 0u64;

    // Read from stdin
    let stdin = tokio::io::stdin();
    let reader = BufReader::new(stdin);
    let mut lines = reader.lines();

    info!("Reading DNS traffic from stdin...");

    loop {
        tokio::select! {
            // Handle shutdown signals
            _ = tokio::signal::ctrl_c() => {
                info!("Received interrupt signal");
                break;
            }

            // Process stdin lines
            line_result = lines.next_line() => {
                match line_result {
                    Ok(Some(line)) => {
                        process_line(&mut dns_parser, db, args, &line, &mut cleanup_counter).await;
                    }
                    Ok(None) => {
                        // EOF reached
                        info!("End of input");
                        break;
                    }
                    Err(e) => {
                        warn!("Error reading line: {}", e);
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}

async fn process_line(
    dns_parser: &mut DnsParser,
    db: &Arc<RouteDb>,
    args: &Args,
    line: &str,
    cleanup_counter: &mut u64,
) {
    // Parse the line
    if let Some(resolution) = dns_parser.parse_line(line) {
        // Check if domain matches patterns
        if matches_patterns(&resolution.domain, &args.patterns) {
            info!(
                "Matched domain: {} -> {:?}",
                resolution.domain, resolution.ips
            );

            // Add routes for each IP in parallel
            let futures: Vec<_> = resolution.ips.into_iter().map(|ip| {
                let db = Arc::clone(db);
                let domain = resolution.domain.clone();
                let gateway = args.gateway;
                let ttl = args.ttl;

                async move {
                    if let Err(e) = add_route(ip, gateway, &db).await {
                        warn!("Failed to add route for {}: {}", ip, e);
                        return;
                    }
                    if let Err(e) = db.upsert_route(ip, &domain, gateway, ttl).await {
                        warn!("Failed to upsert route for {}: {}", ip, e);
                    }
                }
            }).collect();

            join_all(futures).await;
        }
    }

    // Periodic cleanup of stale pending queries (every 1000 lines)
    *cleanup_counter += 1;
    if *cleanup_counter % 1000 == 0 {
        dns_parser.cleanup_old_queries(60);
    }
}
