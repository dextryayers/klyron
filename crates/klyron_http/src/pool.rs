use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::Mutex;
use tokio::net::TcpStream;
use tokio::sync::Semaphore;
use tracing::{debug, trace};

struct PooledConnection {
    stream: TcpStream,
    host: String,
    created: Instant,
    last_used: Instant,
    idle: bool,
    http2: bool,
}

impl PooledConnection {
    fn is_expired(&self, timeout: Duration) -> bool {
        self.last_used.elapsed() > timeout
    }
}

struct PoolInner {
    connections: HashMap<String, Vec<PooledConnection>>,
    max_per_host: usize,
    idle_timeout: Duration,
}

pub struct ConnectionPool {
    inner: Arc<Mutex<PoolInner>>,
    semaphore: Arc<Semaphore>,
}

impl ConnectionPool {
    pub fn new(max_per_host: usize, max_total: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(PoolInner {
                connections: HashMap::new(),
                max_per_host,
                idle_timeout: Duration::from_secs(30),
            })),
            semaphore: Arc::new(Semaphore::new(max_total)),
        }
    }

    pub async fn get(&self, host: &str) -> Option<TcpStream> {
        let _permit = self.semaphore.acquire().await.ok()?;
        let mut inner = self.inner.lock();
        if let Some(conns) = inner.connections.get_mut(host) {
            while let Some(mut conn) = conns.pop() {
                if conn.is_expired(inner.idle_timeout) {
                    trace!("Dropping expired connection to {}", host);
                    continue;
                }
                conn.idle = false;
                conn.last_used = Instant::now();
                if let Ok(()) = set_tcp_nodelay(&conn.stream) {
                    debug!("Reusing pooled connection to {}", host);
                    drop(_permit);
                    return Some(conn.stream);
                }
            }
        }
        None
    }

    pub async fn put(&self, host: String, stream: TcpStream) {
        let mut inner = self.inner.lock();
        let conns = inner.connections.entry(host.clone()).or_default();
        if conns.len() < inner.max_per_host {
            let _ = set_tcp_nodelay(&stream);
            conns.push(PooledConnection {
                stream,
                host,
                created: Instant::now(),
                last_used: Instant::now(),
                idle: true,
                http2: false,
            });
            debug!("Returned connection to pool");
        }
    }

    pub fn release(&self, _host: &str) {
    }

    pub fn close(&self) {
        let mut inner = self.inner.lock();
        inner.connections.clear();
    }

    pub fn close_host(&self, host: &str) {
        let mut inner = self.inner.lock();
        inner.connections.remove(host);
    }

    pub fn active_count(&self) -> usize {
        let inner = self.inner.lock();
        inner.connections.values().map(|v| v.len()).sum()
    }

    pub fn evict_expired(&self) -> usize {
        let mut inner = self.inner.lock();
        let mut evicted = 0;
        for conns in inner.connections.values_mut() {
            let before = conns.len();
            conns.retain(|c| !c.is_expired(inner.idle_timeout));
            evicted += before - conns.len();
        }
        evicted
    }
}

fn set_tcp_nodelay(stream: &TcpStream) -> std::io::Result<()> {
    stream.set_nodelay(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_new() {
        let pool = ConnectionPool::new(6, 100);
        assert_eq!(pool.active_count(), 0);
    }

    #[test]
    fn test_evict_expired_empty() {
        let pool = ConnectionPool::new(6, 100);
        assert_eq!(pool.evict_expired(), 0);
    }
}
