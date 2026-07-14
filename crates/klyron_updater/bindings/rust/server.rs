//! Server for klyron_updater

pub struct Klyron::UpdaterServer {
    addr: String,
}

impl Klyron::UpdaterServer {
    pub fn new(addr: impl Into<String>) -> Self {
        Self { addr: addr.into() }
    }

    pub fn start(&self) -> anyhow::Result<()> {
        println!("{} server starting on {}", "klyron_updater", self.addr);
        Ok(())
    }

    pub fn stop(&self) -> anyhow::Result<()> {
        Ok(())
    }
}
