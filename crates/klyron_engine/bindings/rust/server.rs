//! Server for klyron_engine

pub struct Klyron::EngineServer {
    addr: String,
}

impl Klyron::EngineServer {
    pub fn new(addr: impl Into<String>) -> Self {
        Self { addr: addr.into() }
    }

    pub fn start(&self) -> anyhow::Result<()> {
        println!("{} server starting on {}", "klyron_engine", self.addr);
        Ok(())
    }

    pub fn stop(&self) -> anyhow::Result<()> {
        Ok(())
    }
}
