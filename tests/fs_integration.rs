/// Integration tests for klyron filesystem operations.

#[cfg(test)]
mod tests {
    use std::path::Path;
    use klyron_fs::FileSystem;

    #[test]
    fn test_fs_new() {
        let fs = FileSystem::new();
        let tmp = std::env::temp_dir().join("klyron_test_fs");
        let _ = std::fs::create_dir_all(&tmp);
        let test_file = tmp.join("hello.txt");
        std::fs::write(&test_file, b"world").unwrap();
        let content = fs.read_string_sync(&test_file).unwrap();
        assert_eq!(content, "world");
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_fs_read_nonexistent() {
        let fs = FileSystem::new();
        let result = fs.read_string_sync(Path::new("/nonexistent/path/file.txt"));
        assert!(result.is_err());
    }
}
