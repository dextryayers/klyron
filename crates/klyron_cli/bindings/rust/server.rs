//! Server for klyron_cli

pub struct Klyron::CliServer {
    addr: String,
}

impl Klyron::CliServer {
    pub fn new(addr: impl Into<String>) -> Self {
        Self { addr: addr.into() }
    }

    pub fn start(&self) -> anyhow::Result<()> {
        println!("{} server starting on {}", "klyron_cli", self.addr);
        Ok(())
    }

    pub fn stop(&self) -> anyhow::Result<()> {
        Ok(())
    }
}
