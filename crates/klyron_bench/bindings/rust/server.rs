use std::sync::{Arc, Mutex};

pub struct BenchServer {
    clients: Arc<Mutex<Vec<crate::client::BenchClient>>>,
}

impl BenchServer {
    pub fn new() -> Self { Self { clients: Arc::new(Mutex::new(Vec::new())) } }
    pub fn add_client(&self, client: crate::client::BenchClient) {
        self.clients.lock().unwrap().push(client);
    }
    pub fn client_count(&self) -> usize { self.clients.lock().unwrap().len() }
}
