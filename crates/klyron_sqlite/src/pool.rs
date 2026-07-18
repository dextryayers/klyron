use std::collections::VecDeque;
use std::path::Path;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use rusqlite::Connection;
use thiserror::Error;



#[derive(Error, Debug)]
pub enum PoolError {
    #[error("rusqlite error: {0}")]
    Rusqlite(#[from] rusqlite::Error),

    #[error("pool exhausted: all {0} connections in use")]
    Exhausted(usize),

    #[error("mutex poisoned")]
    MutexPoisoned,

    #[error("connection timeout after {0:?}")]
    Timeout(Duration),
}

struct PooledConnection {
    conn: Connection,
    created_at: Instant,
    last_used: Instant,
}

pub struct SqlitePool {
    connections: Mutex<VecDeque<PooledConnection>>,
    max_size: usize,
    min_idle: usize,
    max_lifetime: Duration,
    idle_timeout: Duration,
    conn_string: String,
    created: Mutex<usize>,
}

impl SqlitePool {
    pub fn new(path: &Path, max_size: usize) -> Result<Self, PoolError> {
        let pool = Self {
            connections: Mutex::new(VecDeque::new()),
            max_size,
            min_idle: 1,
            max_lifetime: Duration::from_secs(1800),
            idle_timeout: Duration::from_secs(600),
            conn_string: path.display().to_string(),
            created: Mutex::new(0),
        };
        pool.warmup()?;
        Ok(pool)
    }

    pub fn new_in_memory(max_size: usize) -> Result<Self, PoolError> {
        let pool = Self {
            connections: Mutex::new(VecDeque::new()),
            max_size,
            min_idle: 1,
            max_lifetime: Duration::from_secs(1800),
            idle_timeout: Duration::from_secs(600),
            conn_string: ":memory:".to_string(),
            created: Mutex::new(0),
        };
        pool.warmup()?;
        Ok(pool)
    }

    pub fn with_min_idle(mut self, min_idle: usize) -> Self {
        self.min_idle = min_idle;
        self
    }

    pub fn with_max_lifetime(mut self, lifetime: Duration) -> Self {
        self.max_lifetime = lifetime;
        self
    }

    pub fn with_idle_timeout(mut self, timeout: Duration) -> Self {
        self.idle_timeout = timeout;
        self
    }

    fn warmup(&self) -> Result<(), PoolError> {
        for _ in 0..self.min_idle {
            self.create_connection()?;
        }
        Ok(())
    }

    fn create_connection(&self) -> Result<(), PoolError> {
        let conn = if self.conn_string == ":memory:" {
            Connection::open_in_memory()?
        } else {
            Connection::open(&self.conn_string)?
        };
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        conn.set_prepared_statement_cache_capacity(64);

        let mut connections = self.connections.lock().map_err(|_| PoolError::MutexPoisoned)?;
        connections.push_back(PooledConnection {
            conn,
            created_at: Instant::now(),
            last_used: Instant::now(),
        });
        *self.created.lock().map_err(|_| PoolError::MutexPoisoned)? += 1;
        Ok(())
    }

    pub fn acquire(&self) -> Result<PoolGuard, PoolError> {
        let mut connections = self.connections.lock().map_err(|_| PoolError::MutexPoisoned)?;

        self.evict_stale(&mut connections);

        if let Some(pc) = connections.pop_front() {
            return Ok(PoolGuard {
                conn: Some(pc.conn),
                pool: self as *const Self as *mut Self,
            });
        }

        let created = *self.created.lock().unwrap_or_else(|e| e.into_inner());
        if created < self.max_size {
            let conn = if self.conn_string == ":memory:" {
                Connection::open_in_memory()?
            } else {
                Connection::open(&self.conn_string)?
            };
            conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
            conn.set_prepared_statement_cache_capacity(64);
            *self.created.lock().unwrap_or_else(|e| e.into_inner()) += 1;
            return Ok(PoolGuard {
                conn: Some(conn),
                pool: self as *const Self as *mut Self,
            });
        }

        Err(PoolError::Exhausted(self.max_size))
    }

    pub fn acquire_timeout(&self, timeout: Duration) -> Result<PoolGuard, PoolError> {
        let start = Instant::now();
        loop {
            match self.acquire() {
                Ok(guard) => return Ok(guard),
                Err(PoolError::Exhausted(_)) => {
                    if start.elapsed() >= timeout {
                        return Err(PoolError::Timeout(timeout));
                    }
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(e) => return Err(e),
            }
        }
    }

    fn evict_stale(&self, connections: &mut VecDeque<PooledConnection>) {
        let now = Instant::now();
        connections.retain(|pc| {
            let alive = now.duration_since(pc.created_at) < self.max_lifetime
                && now.duration_since(pc.last_used) < self.idle_timeout;
            if !alive {
                *self.created.lock().unwrap() -= 1;
            }
            alive
        });
    }

    fn release(&self, conn: Connection) {
        let mut connections = self.connections.lock().unwrap();
        connections.push_back(PooledConnection {
            conn,
            created_at: Instant::now(),
            last_used: Instant::now(),
        });
    }

    pub fn size(&self) -> usize {
        self.connections.lock().map(|c| c.len()).unwrap_or(0)
    }

    pub fn max_size(&self) -> usize {
        self.max_size
    }

    pub fn status(&self) -> String {
        let available = self.connections.lock().map(|c| c.len()).unwrap_or(0);
        let created = *self.created.lock().unwrap_or_else(|e| e.into_inner());
        format!("Pool: {}/{} available, {} total created", available, self.max_size, created)
    }
}

pub struct PoolGuard {
    conn: Option<Connection>,
    pool: *mut SqlitePool,
}

impl PoolGuard {
    pub fn conn(&self) -> &Connection {
        self.conn.as_ref().unwrap()
    }

    pub fn conn_mut(&mut self) -> &mut Connection {
        self.conn.as_mut().unwrap()
    }
}

impl Drop for PoolGuard {
    fn drop(&mut self) {
        if let Some(conn) = self.conn.take() {
            unsafe {
                (*self.pool).release(conn);
            }
        }
    }
}

unsafe impl Send for PoolGuard {}
unsafe impl Sync for PoolGuard {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_create() {
        let pool = SqlitePool::new_in_memory(5).unwrap();
        assert_eq!(pool.max_size(), 5);
    }

    #[test]
    fn test_pool_acquire_release() {
        let pool = SqlitePool::new_in_memory(5).unwrap();
        let guard = pool.acquire().unwrap();
        guard.conn().execute_batch("CREATE TABLE t (id INTEGER)").unwrap();
        drop(guard);
        assert!(pool.size() >= 1);
    }

    #[test]
    fn test_pool_execute_query() {
        let pool = SqlitePool::new_in_memory(3).unwrap();
        let guard = pool.acquire().unwrap();
        guard.conn().execute_batch("CREATE TABLE t (v INTEGER)").unwrap();
        guard.conn().execute("INSERT INTO t VALUES (42)", []).unwrap();
        let val: i64 = guard.conn().query_row("SELECT v FROM t", [], |r| r.get(0)).unwrap();
        assert_eq!(val, 42);
    }

    #[test]
    fn test_pool_max_connections() {
        let pool = SqlitePool::new_in_memory(2).unwrap();
        let _g1 = pool.acquire().unwrap();
        let _g2 = pool.acquire().unwrap();
        let result = pool.acquire_timeout(Duration::from_millis(100));
        assert!(result.is_err());
    }

    #[test]
    fn test_pool_status() {
        let pool = SqlitePool::new_in_memory(3).unwrap();
        let status = pool.status();
        assert!(status.contains("Pool:"));
    }
}
