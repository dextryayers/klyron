use std::sync::{Arc, Mutex};

pub struct BundlerServer {
    clients: Arc<Mutex<Vec<crate::client::BundlerClient>>>,
}

impl BundlerServer {
    pub fn new() -> Self { Self { clients: Arc::new(Mutex::new(Vec::new())) } }
    pub fn add_client(&self, client: crate::client::BundlerClient) {
        self.clients.lock().unwrap().push(client);
    }
    pub fn client_count(&self) -> usize { self.clients.lock().unwrap().len() }
}
