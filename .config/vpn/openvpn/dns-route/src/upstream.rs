use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::Arc;

use anyhow::Result;
use tokio::net::UdpSocket;
use tokio::sync::{oneshot, Mutex, RwLock};
use tokio::time::{timeout, Duration};
use tracing::warn;

const MAX_PACKET_SIZE: usize = 4096;

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

    fn register(&mut self, internal_txid: u16, original_txid: u16) -> oneshot::Receiver<Vec<u8>> {
        let (tx, rx) = oneshot::channel();
        self.queries.insert(internal_txid, (original_txid, tx));
        rx
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
        let server_socket = Arc::new(Self {
            socket,
            server_addr,
            pending,
            next_txid: AtomicU16::new(0),
        });

        server_socket.clone().spawn_receiver();
        Ok(server_socket)
    }

    fn spawn_receiver(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut buf = [0u8; MAX_PACKET_SIZE];
            loop {
                match self.socket.recv(&mut buf).await {
                    Ok(len) => {
                        self.pending.lock().await.dispatch(&buf[..len]);
                    }
                    Err(e) => {
                        warn!("Receiver error for {}: {}", self.server_addr, e);
                        break;
                    }
                }
            }
        });
    }

    async fn query(&self, query: &[u8], timeout_duration: Duration) -> Result<Vec<u8>> {
        if query.len() < 2 {
            anyhow::bail!("Query too short");
        }
        let original_txid = u16::from_be_bytes([query[0], query[1]]);

        // Generate unique internal txid to avoid collisions from concurrent queries
        let internal_txid = self.next_txid.fetch_add(1, Ordering::Relaxed);

        // Rewrite query with internal txid
        let mut rewritten_query = query.to_vec();
        rewritten_query[0..2].copy_from_slice(&internal_txid.to_be_bytes());

        let rx = self.pending.lock().await.register(internal_txid, original_txid);
        self.socket.send(&rewritten_query).await?;

        match timeout(timeout_duration, rx).await {
            Ok(Ok(response)) => Ok(response), // original txid already restored in dispatch()
            Ok(Err(_)) => anyhow::bail!("Response channel closed"),
            Err(_) => {
                self.pending.lock().await.remove(internal_txid);
                anyhow::bail!("Query timeout")
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
                self.sockets.write().await.remove(&server);
                Err(e)
            }
        }
    }
}
