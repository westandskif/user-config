use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use std::sync::Arc;

use anyhow::Result;
use tokio::net::UdpSocket;
use tokio::sync::{oneshot, watch, Mutex, RwLock};
use tokio::time::{timeout, Duration};
use tracing::warn;

const MAX_PACKET_SIZE: usize = 4096;

/// Error types for upstream DNS queries
#[derive(Debug)]
pub enum QueryError {
    // Non-fatal - socket is still usable
    Timeout,
    NoAvailableTxids,
    InvalidQuery,
    // Fatal - socket should be evicted
    SendFailed(std::io::Error),
    ReceiverDied,
}

impl QueryError {
    pub fn is_fatal(&self) -> bool {
        matches!(self, QueryError::SendFailed(_) | QueryError::ReceiverDied)
    }
}

impl std::fmt::Display for QueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryError::Timeout => write!(f, "Query timeout"),
            QueryError::NoAvailableTxids => write!(f, "No available transaction IDs"),
            QueryError::InvalidQuery => write!(f, "Query too short"),
            QueryError::SendFailed(e) => write!(f, "Send failed: {}", e),
            QueryError::ReceiverDied => write!(f, "Response channel closed"),
        }
    }
}

impl std::error::Error for QueryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            QueryError::SendFailed(e) => Some(e),
            _ => None,
        }
    }
}

/// Routes responses to waiting queries by internal transaction ID
struct PendingQueries {
    /// Maps internal_txid -> (original_txid, sender)
    queries: HashMap<u16, (u16, oneshot::Sender<Vec<u8>>)>,
}

impl PendingQueries {
    fn new() -> Self {
        Self {
            queries: HashMap::new(),
        }
    }

    fn register(&mut self, internal_txid: u16, original_txid: u16) -> Option<oneshot::Receiver<Vec<u8>>> {
        if self.queries.contains_key(&internal_txid) {
            return None; // Collision detected
        }
        let (tx, rx) = oneshot::channel();
        self.queries.insert(internal_txid, (original_txid, tx));
        Some(rx)
    }

    fn dispatch(&mut self, response: &[u8]) -> bool {
        if response.len() < 2 {
            return false;
        }
        let internal_txid = u16::from_be_bytes([response[0], response[1]]);
        if let Some((original_txid, sender)) = self.queries.remove(&internal_txid) {
            // Restore original txid in response
            let mut response = response.to_vec();
            response[0..2].copy_from_slice(&original_txid.to_be_bytes());
            let _ = sender.send(response);
            true
        } else {
            false
        }
    }

    fn remove(&mut self, internal_txid: u16) {
        self.queries.remove(&internal_txid);
    }
}

/// Persistent socket to one upstream server
struct ServerSocket {
    socket: Arc<UdpSocket>,
    server_addr: SocketAddr,
    pending: Arc<Mutex<PendingQueries>>,
    /// Counter for generating unique internal transaction IDs
    next_txid: AtomicU16,
    /// Shutdown signal for receiver task
    shutdown_tx: watch::Sender<()>,
    /// Tracks whether receiver task is running
    alive: AtomicBool,
}

impl ServerSocket {
    async fn new(server_addr: SocketAddr) -> Result<Arc<Self>> {
        let bind_addr: SocketAddr = if server_addr.is_ipv6() {
            SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), 0)
        } else {
            SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0)
        };

        let socket = Arc::new(UdpSocket::bind(bind_addr).await?);
        socket.connect(server_addr).await?;

        let pending = Arc::new(Mutex::new(PendingQueries::new()));
        let (shutdown_tx, shutdown_rx) = watch::channel(());
        let server_socket = Arc::new(Self {
            socket,
            server_addr,
            pending,
            next_txid: AtomicU16::new(0),
            shutdown_tx,
            alive: AtomicBool::new(true),
        });

        server_socket.clone().spawn_receiver(shutdown_rx);
        Ok(server_socket)
    }

    fn spawn_receiver(self: Arc<Self>, mut shutdown_rx: watch::Receiver<()>) {
        tokio::spawn(async move {
            let mut buf = [0u8; MAX_PACKET_SIZE];
            loop {
                tokio::select! {
                    result = self.socket.recv(&mut buf) => {
                        match result {
                            Ok(len) => {
                                self.pending.lock().await.dispatch(&buf[..len]);
                            }
                            Err(e) => {
                                warn!("Receiver error for {}: {}", self.server_addr, e);
                                self.alive.store(false, Ordering::Release);
                                break;
                            }
                        }
                    }
                    _ = shutdown_rx.changed() => {
                        self.alive.store(false, Ordering::Release);
                        break;
                    }
                }
            }
        });
    }

    fn shutdown(&self) {
        let _ = self.shutdown_tx.send(());
    }

    async fn query(&self, query: &[u8], timeout_duration: Duration) -> Result<Vec<u8>, QueryError> {
        if !self.alive.load(Ordering::Acquire) {
            return Err(QueryError::ReceiverDied);
        }
        if query.len() < 2 {
            return Err(QueryError::InvalidQuery);
        }
        let original_txid = u16::from_be_bytes([query[0], query[1]]);

        // Allocate internal txid with collision detection
        let (internal_txid, rx) = {
            let mut pending = self.pending.lock().await;
            let mut attempts = 0u32;
            loop {
                let txid = self.next_txid.fetch_add(1, Ordering::Relaxed);
                if let Some(rx) = pending.register(txid, original_txid) {
                    break (txid, rx);
                }
                attempts += 1;
                if attempts >= 65536 {
                    return Err(QueryError::NoAvailableTxids);
                }
            }
        };

        // Rewrite query with internal txid
        let mut rewritten_query = query.to_vec();
        rewritten_query[0..2].copy_from_slice(&internal_txid.to_be_bytes());
        if let Err(e) = self.socket.send(&rewritten_query).await {
            self.pending.lock().await.remove(internal_txid);
            return Err(QueryError::SendFailed(e));
        }

        match timeout(timeout_duration, rx).await {
            Ok(Ok(response)) => Ok(response), // original txid already restored in dispatch()
            Ok(Err(_)) => Err(QueryError::ReceiverDied),
            Err(_) => {
                self.pending.lock().await.remove(internal_txid);
                if !self.alive.load(Ordering::Acquire) {
                    Err(QueryError::ReceiverDied)
                } else {
                    Err(QueryError::Timeout)
                }
            }
        }
    }
}

/// Manages persistent sockets to all upstream servers
pub struct UpstreamSocketManager {
    sockets: RwLock<HashMap<IpAddr, Arc<ServerSocket>>>,
}

impl UpstreamSocketManager {
    pub fn new() -> Self {
        Self {
            sockets: RwLock::new(HashMap::new()),
        }
    }

    async fn get_socket(&self, server: IpAddr) -> Result<Arc<ServerSocket>> {
        // Fast path
        if let Some(socket) = self.sockets.read().await.get(&server) {
            return Ok(socket.clone());
        }

        // Slow path: create new socket
        let mut sockets = self.sockets.write().await;
        if let Some(socket) = sockets.get(&server) {
            return Ok(socket.clone());
        }

        let server_addr = SocketAddr::new(server, 53);
        let socket = ServerSocket::new(server_addr).await?;
        sockets.insert(server, socket.clone());
        Ok(socket)
    }

    pub async fn query(
        &self,
        query: &[u8],
        server: IpAddr,
        timeout_duration: Duration,
    ) -> Result<Vec<u8>> {
        let socket = self.get_socket(server).await?;
        match socket.query(query, timeout_duration).await {
            Ok(response) => Ok(response),
            Err(e) => {
                // Only evict on fatal errors that indicate a broken socket
                if e.is_fatal() {
                    if let Some(socket) = self.sockets.write().await.remove(&server) {
                        socket.shutdown();
                    }
                }
                Err(anyhow::Error::new(e))
            }
        }
    }
}
