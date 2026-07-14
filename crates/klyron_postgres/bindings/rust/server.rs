//! Server for klyron_postgres

pub struct Klyron::PostgresServer {
    addr: String,
}

impl Klyron::PostgresServer {
    pub fn new(addr: impl Into<String>) -> Self {
        Self { addr: addr.into() }
    }

    pub fn start(&self) -> anyhow::Result<()> {
        println!("{} server starting on {}", "klyron_postgres", self.addr);
        Ok(())
    }

    pub fn stop(&self) -> anyhow::Result<()> {
        Ok(())
    }
}
