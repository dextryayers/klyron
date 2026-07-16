use std::path::Path;

use deno_core::{extension, op2, Extension};
use deno_error::JsErrorBox;
use serde::Serialize;

extension!(
  klyron_fs,
  ops = [op_fs_read_file, op_fs_write_file, op_fs_mkdir, op_fs_read_dir, op_fs_stat, op_fs_exists, op_fs_remove, op_fs_copy, op_fs_rename],
  esm_entry_point = "ext:klyron_fs/fs.js",
  esm = [dir "js", "fs.js"],
);

pub fn init() -> Extension {
  klyron_fs::init()
}

fn op_fs_read_file_impl(path: String) -> Result<String, JsErrorBox> {
  std::fs::read_to_string(&path).map_err(|e| JsErrorBox::generic(format!("read {path}: {e}")))
}

#[op2]
#[string]
fn op_fs_read_file(#[string] path: String) -> Result<String, JsErrorBox> {
  op_fs_read_file_impl(path)
}

fn op_fs_write_file_impl(path: String, data: String) -> Result<(), JsErrorBox> {
  std::fs::write(&path, &data).map_err(|e| JsErrorBox::generic(format!("write {path}: {e}")))
}

#[op2(fast)]
fn op_fs_write_file(#[string] path: String, #[string] data: String) -> Result<(), JsErrorBox> {
  op_fs_write_file_impl(path, data)
}

fn op_fs_mkdir_impl(path: String) -> Result<(), JsErrorBox> {
  std::fs::create_dir_all(&path).map_err(|e| JsErrorBox::generic(format!("mkdir {path}: {e}")))
}

#[op2(fast)]
fn op_fs_mkdir(#[string] path: String) -> Result<(), JsErrorBox> {
  op_fs_mkdir_impl(path)
}

#[derive(Serialize)]
struct DirEntry {
  name: String,
  is_file: bool,
  is_dir: bool,
}

fn op_fs_read_dir_impl(path: String) -> Result<Vec<DirEntry>, JsErrorBox> {
  let entries = std::fs::read_dir(&path).map_err(|e| JsErrorBox::generic(format!("readdir {path}: {e}")))?;
  let mut result = Vec::new();
  for entry in entries {
    if let Ok(entry) = entry {
      let ft = entry.file_type().ok();
      result.push(DirEntry {
        name: entry.file_name().to_string_lossy().to_string(),
        is_file: ft.as_ref().map(|f| f.is_file()).unwrap_or(false),
        is_dir: ft.as_ref().map(|f| f.is_dir()).unwrap_or(false),
      });
    }
  }
  Ok(result)
}

#[op2]
#[serde]
fn op_fs_read_dir(#[string] path: String) -> Result<Vec<DirEntry>, JsErrorBox> {
  op_fs_read_dir_impl(path)
}

#[derive(Serialize)]
struct FileInfo {
  is_file: bool,
  is_dir: bool,
  size: u64,
  modified: String,
}

fn op_fs_stat_impl(path: String) -> Result<FileInfo, JsErrorBox> {
  let meta = std::fs::metadata(&path).map_err(|e| JsErrorBox::generic(format!("stat {path}: {e}")))?;
  Ok(FileInfo {
    is_file: meta.is_file(),
    is_dir: meta.is_dir(),
    size: meta.len(),
    modified: meta.modified().map(|t| format!("{:?}", t)).unwrap_or_default(),
  })
}

#[op2]
#[serde]
fn op_fs_stat(#[string] path: String) -> Result<FileInfo, JsErrorBox> {
  op_fs_stat_impl(path)
}

fn op_fs_exists_impl(path: String) -> bool {
  Path::new(&path).exists()
}

#[op2(fast)]
fn op_fs_exists(#[string] path: String) -> bool {
  op_fs_exists_impl(path)
}

fn op_fs_remove_impl(path: String) -> Result<(), JsErrorBox> {
  let p = Path::new(&path);
  if p.is_dir() {
    std::fs::remove_dir_all(p).map_err(|e| JsErrorBox::generic(format!("remove_dir {path}: {e}")))
  } else {
    std::fs::remove_file(p).map_err(|e| JsErrorBox::generic(format!("remove_file {path}: {e}")))
  }
}

#[op2(fast)]
fn op_fs_remove(#[string] path: String) -> Result<(), JsErrorBox> {
  op_fs_remove_impl(path)
}

fn op_fs_copy_impl(src: String, dest: String) -> Result<(), JsErrorBox> {
  std::fs::copy(&src, &dest).map_err(|e| JsErrorBox::generic(format!("copy {src} -> {dest}: {e}")))?;
  Ok(())
}

#[op2(fast)]
fn op_fs_copy(#[string] src: String, #[string] dest: String) -> Result<(), JsErrorBox> {
  op_fs_copy_impl(src, dest)
}

fn op_fs_rename_impl(src: String, dest: String) -> Result<(), JsErrorBox> {
  std::fs::rename(&src, &dest).map_err(|e| JsErrorBox::generic(format!("rename {src} -> {dest}: {e}")))
}

#[op2(fast)]
fn op_fs_rename(#[string] src: String, #[string] dest: String) -> Result<(), JsErrorBox> {
  op_fs_rename_impl(src, dest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn test_dir() -> std::path::PathBuf {
        let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("klyron_ext_fs_test_{id}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_init_returns_extension() {
        let ext = init();
        assert_eq!(ext.name, "klyron_fs");
    }

    #[test]
    fn test_fs_write_and_read() {
        let dir = test_dir();
        let path = dir.join("test.txt").to_string_lossy().to_string();
        op_fs_write_file_impl(path.clone(), "hello world".to_string()).unwrap();
        let content = op_fs_read_file_impl(path).unwrap();
        assert_eq!(content, "hello world");
    }

    #[test]
    fn test_fs_read_nonexistent() {
        let result = op_fs_read_file_impl("/tmp/nonexistent_file_xyz_12345".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_fs_mkdir() {
        let dir = test_dir();
        let sub = dir.join("subdir").to_string_lossy().to_string();
        op_fs_mkdir_impl(sub.clone()).unwrap();
        assert!(std::path::Path::new(&sub).exists());
    }

    #[test]
    fn test_fs_exists_true() {
        assert!(op_fs_exists_impl("/tmp".to_string()));
    }

    #[test]
    fn test_fs_exists_false() {
        assert!(!op_fs_exists_impl("/nonexistent_path_xyz_99999".to_string()));
    }

    #[test]
    fn test_fs_stat_file() {
        let dir = test_dir();
        let path = dir.join("stat.txt").to_string_lossy().to_string();
        op_fs_write_file_impl(path.clone(), "data".to_string()).unwrap();
        let info = op_fs_stat_impl(path).unwrap();
        assert!(info.is_file);
        assert!(!info.is_dir);
    }

    #[test]
    fn test_fs_stat_dir() {
        let info = op_fs_stat_impl("/tmp".to_string()).unwrap();
        assert!(!info.is_file);
        assert!(info.is_dir);
    }

    #[test]
    fn test_fs_stat_error() {
        let result = op_fs_stat_impl("/nonexistent_xyz_path".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_fs_remove_file() {
        let dir = test_dir();
        let path = dir.join("remove_me.txt").to_string_lossy().to_string();
        op_fs_write_file_impl(path.clone(), "data".to_string()).unwrap();
        op_fs_remove_impl(path.clone()).unwrap();
        assert!(!std::path::Path::new(&path).exists());
    }

    #[test]
    fn test_fs_copy() {
        let dir = test_dir();
        let src = dir.join("src.txt").to_string_lossy().to_string();
        let dst = dir.join("dst.txt").to_string_lossy().to_string();
        op_fs_write_file_impl(src.clone(), "copy data".to_string()).unwrap();
        op_fs_copy_impl(src, dst.clone()).unwrap();
        assert!(std::path::Path::new(&dst).exists());
    }

    #[test]
    fn test_fs_rename() {
        let dir = test_dir();
        let src = dir.join("old.txt").to_string_lossy().to_string();
        let dst = dir.join("new.txt").to_string_lossy().to_string();
        op_fs_write_file_impl(src.clone(), "rename data".to_string()).unwrap();
        op_fs_rename_impl(src, dst.clone()).unwrap();
        assert!(std::path::Path::new(&dst).exists());
    }

    #[test]
    fn test_fs_read_dir() {
        let dir = test_dir();
        op_fs_write_file_impl(dir.join("a.txt").to_string_lossy().to_string(), "a".to_string()).unwrap();
        op_fs_write_file_impl(dir.join("b.txt").to_string_lossy().to_string(), "b".to_string()).unwrap();
        let entries = op_fs_read_dir_impl(dir.to_string_lossy().to_string()).unwrap();
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_fs_write_empty_string() {
        let dir = test_dir();
        let path = dir.join("empty.txt").to_string_lossy().to_string();
        op_fs_write_file_impl(path.clone(), "".to_string()).unwrap();
        let content = op_fs_read_file_impl(path).unwrap();
        assert_eq!(content, "");
    }

    #[test]
    fn test_fs_stat_size() {
        let dir = test_dir();
        let path = dir.join("size.txt").to_string_lossy().to_string();
        op_fs_write_file_impl(path.clone(), "12345".to_string()).unwrap();
        let info = op_fs_stat_impl(path).unwrap();
        assert_eq!(info.size, 5);
    }
}
