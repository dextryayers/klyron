//! Server for klyron_sqlite

pub struct Klyron::SqliteServer {
    addr: String,
}

impl Klyron::SqliteServer {
    pub fn new(addr: impl Into<String>) -> Self {
        Self { addr: addr.into() }
    }

    pub fn start(&self) -> anyhow::Result<()> {
        println!("{} server starting on {}", "klyron_sqlite", self.addr);
        Ok(())
    }

    pub fn stop(&self) -> anyhow::Result<()> {
        Ok(())
    }
}
