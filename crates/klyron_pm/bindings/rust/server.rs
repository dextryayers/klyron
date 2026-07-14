use crate::{PackageManager, detect, install_cmd};
use std::sync::{Arc, Mutex};
use std::path::PathBuf;

pub struct PmServer {
    clients: Arc<Mutex<Vec<super::PackageManagerClient>>>,
}

impl PmServer {
    pub fn new() -> Self { Self { clients: Arc::new(Mutex::new(Vec::new())) } }

    pub fn add_client(&self, client: super::PackageManagerClient) {
        self.clients.lock().unwrap().push(client);
    }

    pub fn detect_and_add(&self, path: PathBuf) {
        let pm = detect(&path);
        self.add_client(super::PackageManagerClient::new(pm, path));
    }

    pub fn client_count(&self) -> usize {
        self.clients.lock().unwrap().len()
    }
}
