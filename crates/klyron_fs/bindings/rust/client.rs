use crate::types;
use crate::errors::FileSystemError;

pub struct FileSystemClient;
impl FileSystemClient {
    pub fn new() -> Self { Self }
}
impl Default for FileSystemClient { fn default() -> Self { Self::new() } }
