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

#[op2]
#[string]
fn op_fs_read_file(#[string] path: String) -> Result<String, JsErrorBox> {
  std::fs::read_to_string(&path).map_err(|e| JsErrorBox::generic(format!("read {path}: {e}")))
}

#[op2(fast)]
fn op_fs_write_file(#[string] path: String, #[string] data: String) -> Result<(), JsErrorBox> {
  std::fs::write(&path, &data).map_err(|e| JsErrorBox::generic(format!("write {path}: {e}")))
}

#[op2(fast)]
fn op_fs_mkdir(#[string] path: String) -> Result<(), JsErrorBox> {
  std::fs::create_dir_all(&path).map_err(|e| JsErrorBox::generic(format!("mkdir {path}: {e}")))
}

#[derive(Serialize)]
struct DirEntry {
  name: String,
  is_file: bool,
  is_dir: bool,
}

#[op2]
#[serde]
fn op_fs_read_dir(#[string] path: String) -> Result<Vec<DirEntry>, JsErrorBox> {
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

#[derive(Serialize)]
struct FileInfo {
  is_file: bool,
  is_dir: bool,
  size: u64,
  modified: String,
}

#[op2]
#[serde]
fn op_fs_stat(#[string] path: String) -> Result<FileInfo, JsErrorBox> {
  let meta = std::fs::metadata(&path).map_err(|e| JsErrorBox::generic(format!("stat {path}: {e}")))?;
  Ok(FileInfo {
    is_file: meta.is_file(),
    is_dir: meta.is_dir(),
    size: meta.len(),
    modified: meta.modified().map(|t| format!("{:?}", t)).unwrap_or_default(),
  })
}

#[op2(fast)]
fn op_fs_exists(#[string] path: String) -> bool {
  Path::new(&path).exists()
}

#[op2(fast)]
fn op_fs_remove(#[string] path: String) -> Result<(), JsErrorBox> {
  let p = Path::new(&path);
  if p.is_dir() {
    std::fs::remove_dir_all(p).map_err(|e| JsErrorBox::generic(format!("remove_dir {path}: {e}")))
  } else {
    std::fs::remove_file(p).map_err(|e| JsErrorBox::generic(format!("remove_file {path}: {e}")))
  }
}

#[op2(fast)]
fn op_fs_copy(#[string] src: String, #[string] dest: String) -> Result<(), JsErrorBox> {
  std::fs::copy(&src, &dest).map_err(|e| JsErrorBox::generic(format!("copy {src} -> {dest}: {e}")))?;
  Ok(())
}

#[op2(fast)]
fn op_fs_rename(#[string] src: String, #[string] dest: String) -> Result<(), JsErrorBox> {
  std::fs::rename(&src, &dest).map_err(|e| JsErrorBox::generic(format!("rename {src} -> {dest}: {e}")))
}
