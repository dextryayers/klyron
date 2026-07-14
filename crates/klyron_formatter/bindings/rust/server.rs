use std::sync::{Arc, Mutex};

pub struct FormatterServer {
    clients: Arc<Mutex<Vec<crate::client::FormatterClient>>>,
}

impl FormatterServer {
    pub fn new() -> Self { Self { clients: Arc::new(Mutex::new(Vec::new())) } }
    pub fn add_client(&self, client: crate::client::FormatterClient) {
        self.clients.lock().unwrap().push(client);
    }
    pub fn client_count(&self) -> usize { self.clients.lock().unwrap().len() }
}
