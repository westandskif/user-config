use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::net::Ipv4Addr;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct RouteDb {
    pool: Pool<Sqlite>,
}

#[derive(Debug)]
pub struct RouteEntry {
    pub ip: Ipv4Addr,
    pub domain: String,
    pub gateway: Ipv4Addr,
    pub expires_at: i64,
}

impl RouteDb {
    pub async fn new(db_path: &str) -> Result<Self, sqlx::Error> {
        let db_url = format!("sqlite:{}?mode=rwc", db_path);
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await?;

        // Create tables if not exists
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS routes (
                ip TEXT PRIMARY KEY,
                domain TEXT NOT NULL,
                gateway TEXT NOT NULL,
                added_at INTEGER NOT NULL,
                expires_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS pf_exceptions (
                ip TEXT PRIMARY KEY,
                added_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }

    /// Insert or update a route entry. Returns true if this was a new insert.
    pub async fn upsert_route(
        &self,
        ip: Ipv4Addr,
        domain: &str,
        gateway: Ipv4Addr,
        ttl_secs: u64,
    ) -> Result<bool, sqlx::Error> {
        // Check if route already exists
        let exists: Option<(i32,)> = sqlx::query_as("SELECT 1 FROM routes WHERE ip = ?")
            .bind(ip.to_string())
            .fetch_optional(&self.pool)
            .await?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let expires_at = now + ttl_secs as i64;

        sqlx::query(
            r#"
            INSERT INTO routes (ip, domain, gateway, added_at, expires_at)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(ip) DO UPDATE SET
                domain = excluded.domain,
                gateway = excluded.gateway,
                expires_at = excluded.expires_at
            "#,
        )
        .bind(ip.to_string())
        .bind(domain)
        .bind(gateway.to_string())
        .bind(now)
        .bind(expires_at)
        .execute(&self.pool)
        .await?;

        // Returns true if this was a new insert (not an update)
        Ok(exists.is_none())
    }

    /// Get all expired routes
    pub async fn get_expired_routes(&self) -> Result<Vec<RouteEntry>, sqlx::Error> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let rows: Vec<(String, String, String, i64)> = sqlx::query_as(
            "SELECT ip, domain, gateway, expires_at FROM routes WHERE expires_at < ?",
        )
        .bind(now)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .filter_map(|(ip, domain, gateway, expires_at)| {
                Some(RouteEntry {
                    ip: ip.parse().ok()?,
                    domain,
                    gateway: gateway.parse().ok()?,
                    expires_at,
                })
            })
            .collect())
    }

    /// Delete a route from the database
    pub async fn delete_route(&self, ip: Ipv4Addr) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM routes WHERE ip = ?")
            .bind(ip.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Get all routes (for shutdown cleanup)
    pub async fn get_all_routes(&self) -> Result<Vec<RouteEntry>, sqlx::Error> {
        let rows: Vec<(String, String, String, i64)> =
            sqlx::query_as("SELECT ip, domain, gateway, expires_at FROM routes")
                .fetch_all(&self.pool)
                .await?;

        Ok(rows
            .into_iter()
            .filter_map(|(ip, domain, gateway, expires_at)| {
                Some(RouteEntry {
                    ip: ip.parse().ok()?,
                    domain,
                    gateway: gateway.parse().ok()?,
                    expires_at,
                })
            })
            .collect())
    }

    /// Add a pf exception record
    pub async fn add_pf_exception(&self, ip: Ipv4Addr) -> Result<(), sqlx::Error> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        sqlx::query("INSERT OR IGNORE INTO pf_exceptions (ip, added_at) VALUES (?, ?)")
            .bind(ip.to_string())
            .bind(now)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Delete a pf exception record
    pub async fn delete_pf_exception(&self, ip: Ipv4Addr) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM pf_exceptions WHERE ip = ?")
            .bind(ip.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Get all pf exceptions
    pub async fn get_all_pf_exceptions(&self) -> Result<Vec<Ipv4Addr>, sqlx::Error> {
        let rows: Vec<(String,)> = sqlx::query_as("SELECT ip FROM pf_exceptions")
            .fetch_all(&self.pool)
            .await?;

        Ok(rows
            .into_iter()
            .filter_map(|(ip,)| ip.parse().ok())
            .collect())
    }
}
