/* src/lib.rs */

use serde::{Serialize, de::DeserializeOwned};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time;

pub mod db;
pub mod error;

use crate::error::{PathmapError, Result};
use sqlx::SqlitePool;

/// The main struct for interacting with pathmap.
pub struct Pathmap {
    base_path: PathBuf,
    pools: Arc<Mutex<HashMap<String, SqlitePool>>>,
}

impl Pathmap {
    /// Creates a new Pathmap instance with the default path ("/opt/pathmap/").
    pub fn new() -> Self {
        Pathmap {
            base_path: PathBuf::from("/opt/pathmap/"),
            pools: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Overrides the default base path. This must be called before any other operations.
    pub fn with_base_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.base_path = path.as_ref().to_path_buf();
        self
    }

    /// Initializes a new namespace.
    /// This creates a new SQLite file for the namespace.
    pub async fn init_ns(&self, ns: &str) -> Result<bool> {
        let db_path = self.get_db_path(ns);
        if db_path.exists() {
            return Err(PathmapError::NamespaceAlreadyExists(ns.to_string()));
        }
        let pool = db::connect(&db_path).await?;
        let mut pools = self.pools.lock().await; // Use .await for locking
        pools.insert(ns.to_string(), pool);
        Ok(true)
    }

    /// Deletes a namespace, including its SQLite file.
    pub async fn delete_ns(&self, ns: &str) -> Result<bool> {
        {
            let mut pools = self.pools.lock().await; // Use .await for locking
            if let Some(pool) = pools.remove(ns) {
                pool.close().await;
            }
        }
        let db_path = self.get_db_path(ns);
        if !db_path.exists() {
            return Err(PathmapError::NamespaceNotFound(ns.to_string()));
        }
        std::fs::remove_file(db_path)?;
        Ok(true)
    }

    /// Parses a path string like "namespace::group.key" into (namespace, key).
    fn parse_path<'a>(&self, path: &'a str) -> Result<(&'a str, &'a str)> {
        path.split_once("::")
            .ok_or_else(|| PathmapError::InvalidPath(path.to_string()))
    }

    /// Retrieves a value. The value must be deserializable into the specified type `T`.
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let (ns, key) = self.parse_path(path)?;
        let pool = self.get_pool(ns).await?;
        let raw_value = db::get(&pool, key).await?;
        let value: T = serde_json::from_slice(&raw_value)?;
        Ok(value)
    }

    /// Sets a value. The value will be serialized to JSON.
    /// This operation will fail if the key already exists.
    pub async fn set<T: Serialize>(&self, path: &str, value: T) -> Result<()> {
        let (ns, key) = self.parse_path(path)?;
        let pool = self.get_pool(ns).await?;
        if db::exists(&pool, key).await? {
            return Err(PathmapError::ValueAlreadyExists(key.to_string()));
        }
        let serialized_value = serde_json::to_vec(&value)?;
        db::set(&pool, key, &serialized_value).await
    }

    /// Overwrites a value. If the key does not exist, it will be created.
    /// If it exists, its value will be updated.
    pub async fn overwrite<T: Serialize>(&self, path: &str, value: T) -> Result<()> {
        let (ns, key) = self.parse_path(path)?;
        let pool = self.get_pool_or_init(ns).await?;
        let serialized_value = serde_json::to_vec(&value)?;
        db::overwrite(&pool, key, &serialized_value).await
    }

    /// Deletes a value.
    pub async fn delete(&self, path: &str) -> Result<()> {
        let (ns, key) = self.parse_path(path)?;
        let pool = self.get_pool(ns).await?;
        db::delete(&pool, key).await
    }

    /// Checks if a path (namespace, group, or value) exists.
    pub async fn exists(&self, path: &str) -> Result<bool> {
        if let Ok((ns, key)) = self.parse_path(path) {
            if self.get_db_path(ns).exists() {
                let pool = self.get_pool(ns).await?;
                return db::exists(&pool, key).await;
            }
        } else if self.get_db_path(path).exists() {
            return Ok(true);
        }
        Ok(false)
    }

    /// Manually triggers a cleanup (VACUUM) on a namespace's database.
    pub async fn manual_cleanup(&self, ns: &str) -> Result<()> {
        let pool = self.get_pool(ns).await?;
        db::vacuum(&pool).await
    }

    /// Starts a background task for automatic cleanup.
    pub fn start_background_cleanup(&self, check_interval: Duration, idle_timeout: Duration) {
        let pools = Arc::clone(&self.pools);
        let last_access = Arc::new(Mutex::new(HashMap::<String, time::Instant>::new()));

        tokio::spawn(async move {
            let mut interval = time::interval(check_interval);
            loop {
                interval.tick().await;
                let pools_to_check: Vec<(String, SqlitePool)> = pools
                    .lock()
                    .await
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();

                let mut last_access_guard = last_access.lock().await;

                for (ns, pool) in pools_to_check {
                    let now = time::Instant::now();
                    let last = last_access_guard.entry(ns.clone()).or_insert(now);

                    if now.duration_since(*last) > idle_timeout {
                        println!("Namespace '{}' is idle, performing cleanup...", ns);
                        if let Err(e) = db::vacuum(&pool).await {
                            eprintln!("Error during background cleanup of '{}': {}", ns, e);
                        }
                        *last = now;
                    }
                }
            }
        });
    }

    fn get_db_path(&self, ns: &str) -> PathBuf {
        self.base_path.join(format!("{}.sqlite", ns))
    }

    async fn get_pool(&self, ns: &str) -> Result<SqlitePool> {
        let mut pools = self.pools.lock().await;
        if let Some(pool) = pools.get(ns) {
            return Ok(pool.clone());
        }

        let db_path = self.get_db_path(ns);
        if !db_path.exists() {
            return Err(PathmapError::NamespaceNotFound(ns.to_string()));
        }

        let pool = db::connect(&db_path).await?;
        pools.insert(ns.to_string(), pool.clone());
        Ok(pool)
    }

    async fn get_pool_or_init(&self, ns: &str) -> Result<SqlitePool> {
        match self.get_pool(ns).await {
            Ok(pool) => Ok(pool),
            Err(PathmapError::NamespaceNotFound(_)) => {
                self.init_ns(ns).await?;
                self.get_pool(ns).await
            }
            Err(e) => Err(e),
        }
    }
}

impl Default for Pathmap {
    fn default() -> Self {
        Self::new()
    }
}
