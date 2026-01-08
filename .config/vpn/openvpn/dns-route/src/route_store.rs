use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use tokio_rusqlite::Connection;
use tokio_rusqlite::rusqlite::{self, params};

pub struct RouteStore {
    conn: Connection,
}

#[derive(Debug, Clone)]
pub struct StoredRoute {
    pub ip: String,
    #[allow(dead_code)]
    pub family: i64,
    #[allow(dead_code)]
    pub gateway: String,
}

impl RouteStore {
    pub async fn new(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let display_path = path.display().to_string();
        let conn = Connection::open(path)
            .await
            .with_context(|| format!("Failed to open SQLite database at {}", display_path))?;

        conn.call(|conn| -> Result<(), rusqlite::Error> {
            conn.execute_batch(
                "PRAGMA journal_mode = WAL;
                 PRAGMA synchronous = NORMAL;
                 CREATE TABLE IF NOT EXISTS runs (
                     run_id TEXT PRIMARY KEY,
                     started_at INTEGER NOT NULL,
                     ended_at INTEGER,
                     pid INTEGER
                 );
                 CREATE TABLE IF NOT EXISTS routes (
                     ip TEXT PRIMARY KEY,
                     family INTEGER NOT NULL,
                     gateway TEXT NOT NULL,
                     created_at INTEGER NOT NULL,
                     expires_at INTEGER NOT NULL,
                     last_seen INTEGER NOT NULL,
                     run_id TEXT NOT NULL REFERENCES runs(run_id)
                 );
                 CREATE INDEX IF NOT EXISTS idx_routes_expires_at ON routes(expires_at);"
            )?;
            Ok(())
        })
        .await
        .context("Failed to initialize SQLite schema")?;

        Ok(Self { conn })
    }

    pub async fn init(&self, run_id: &str, pid: u32) -> Result<()> {
        let run_id = run_id.to_string();
        let now = now_unix();
        self.conn
            .call(move |conn| -> Result<(), rusqlite::Error> {
                conn.execute(
                    "INSERT INTO runs (run_id, started_at, pid) VALUES (?1, ?2, ?3)
                     ON CONFLICT(run_id) DO UPDATE SET started_at = excluded.started_at, pid = excluded.pid, ended_at = NULL",
                    params![run_id, now, pid as i64],
                )?;
                Ok(())
            })
            .await
            .context("Failed to record run start")
    }

    pub async fn record_run_end(&self, run_id: &str) -> Result<()> {
        let run_id = run_id.to_string();
        let now = now_unix();
        self.conn
            .call(move |conn| -> Result<(), rusqlite::Error> {
                conn.execute(
                    "UPDATE runs SET ended_at = ?1 WHERE run_id = ?2",
                    params![now, run_id],
                )?;
                Ok(())
            })
            .await
            .context("Failed to record run end")
    }

    pub async fn list_orphan_routes(&self, current_run_id: &str) -> Result<Vec<StoredRoute>> {
        let current_run_id = current_run_id.to_string();
        self.conn
            .call(move |conn| -> Result<Vec<StoredRoute>, rusqlite::Error> {
                let mut stmt = conn.prepare(
                    "SELECT r.ip, r.family, r.gateway
                     FROM routes r
                     JOIN runs ru ON r.run_id = ru.run_id
                     WHERE ru.ended_at IS NULL AND r.run_id != ?1",
                )?;

                let mut rows = stmt.query(params![current_run_id])?;
                let mut routes = Vec::new();
                while let Some(row) = rows.next()? {
                    routes.push(StoredRoute {
                        ip: row.get(0)?,
                        family: row.get(1)?,
                        gateway: row.get(2)?,
                    });
                }
                Ok(routes)
            })
            .await
            .context("Failed to list orphan routes")
    }

    pub async fn list_expired_routes(&self, now: i64, run_id: &str) -> Result<Vec<StoredRoute>> {
        let run_id = run_id.to_string();
        self.conn
            .call(move |conn| -> Result<Vec<StoredRoute>, rusqlite::Error> {
                let mut stmt = conn.prepare(
                    "SELECT ip, family, gateway
                     FROM routes
                     WHERE expires_at <= ?1 AND run_id = ?2",
                )?;

                let mut rows = stmt.query(params![now, run_id])?;
                let mut routes = Vec::new();
                while let Some(row) = rows.next()? {
                    routes.push(StoredRoute {
                        ip: row.get(0)?,
                        family: row.get(1)?,
                        gateway: row.get(2)?,
                    });
                }
                Ok(routes)
            })
            .await
            .context("Failed to list expired routes")
    }

    pub async fn list_run_routes(&self, run_id: &str) -> Result<Vec<StoredRoute>> {
        let run_id = run_id.to_string();
        self.conn
            .call(move |conn| -> Result<Vec<StoredRoute>, rusqlite::Error> {
                let mut stmt = conn.prepare(
                    "SELECT ip, family, gateway
                     FROM routes
                     WHERE run_id = ?1",
                )?;

                let mut rows = stmt.query(params![run_id])?;
                let mut routes = Vec::new();
                while let Some(row) = rows.next()? {
                    routes.push(StoredRoute {
                        ip: row.get(0)?,
                        family: row.get(1)?,
                        gateway: row.get(2)?,
                    });
                }
                Ok(routes)
            })
            .await
            .context("Failed to list run routes")
    }

    pub async fn upsert_route(
        &self,
        ip: &str,
        family: i64,
        gateway: &str,
        expires_at: i64,
        run_id: &str,
    ) -> Result<()> {
        let ip = ip.to_string();
        let gateway = gateway.to_string();
        let run_id = run_id.to_string();
        let now = now_unix();
        self.conn
            .call(move |conn| -> Result<(), rusqlite::Error> {
                conn.execute(
                    "INSERT INTO routes (ip, family, gateway, created_at, expires_at, last_seen, run_id)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                     ON CONFLICT(ip) DO UPDATE SET
                         family = excluded.family,
                         gateway = excluded.gateway,
                         expires_at = excluded.expires_at,
                         last_seen = excluded.last_seen,
                         run_id = excluded.run_id",
                    params![ip, family, gateway, now, expires_at, now, run_id],
                )?;
                Ok(())
            })
            .await
            .context("Failed to upsert route")
    }

    pub async fn route_exists(&self, ip: &str) -> Result<bool> {
        let ip = ip.to_string();
        self.conn
            .call(move |conn| -> Result<bool, rusqlite::Error> {
                let count: i64 = conn.query_row(
                    "SELECT COUNT(*) FROM routes WHERE ip = ?1",
                    params![ip],
                    |row| row.get(0),
                )?;
                Ok(count > 0)
            })
            .await
            .context("Failed to check route existence")
    }

    pub async fn delete_route(&self, ip: &str) -> Result<()> {
        let ip = ip.to_string();
        self.conn
            .call(move |conn| -> Result<(), rusqlite::Error> {
                conn.execute("DELETE FROM routes WHERE ip = ?1", params![ip])?;
                Ok(())
            })
            .await
            .context("Failed to delete route")
    }

    pub async fn clear_run_routes(&self, run_id: &str) -> Result<()> {
        let run_id = run_id.to_string();
        self.conn
            .call(move |conn| -> Result<(), rusqlite::Error> {
                conn.execute("DELETE FROM routes WHERE run_id = ?1", params![run_id])?;
                Ok(())
            })
            .await
            .context("Failed to clear run routes")
    }
}

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}
