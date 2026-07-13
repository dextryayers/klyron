import { op_fs_read_file, op_fs_write_file, op_fs_mkdir, op_fs_read_dir, op_fs_stat, op_fs_exists, op_fs_remove, op_fs_copy, op_fs_rename } from "ext:core/ops";

export async function readFile(path) { return op_fs_read_file(path); }
export async function writeFile(path, data) { return op_fs_write_file(path, data); }
export async function mkdir(path) { return op_fs_mkdir(path); }
export async function readDir(path) { return op_fs_read_dir(path); }
export async function stat(path) { return op_fs_stat(path); }
export async function exists(path) { return op_fs_exists(path); }
export async function remove(path) { return op_fs_remove(path); }
export async function copy(from, to) { return op_fs_copy(from, to); }
export async function rename(from, to) { return op_fs_rename(from, to); }

export default { readFile, writeFile, mkdir, readDir, stat, exists, remove, copy, rename };
