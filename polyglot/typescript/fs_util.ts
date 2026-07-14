import { FileInfo } from "./types";

const nodeFs = await import("fs");
const nodePath = await import("path");

export function readFile(path: string): Buffer {
  return nodeFs.readFileSync(path);
}

export function readFileString(path: string): string {
  return nodeFs.readFileSync(path, "utf-8");
}

export function writeFile(path: string, data: string | Buffer): void {
  nodeFs.mkdirSync(nodePath.dirname(path), { recursive: true });
  nodeFs.writeFileSync(path, data);
}

export function appendFile(path: string, data: string | Buffer): void {
  nodeFs.appendFileSync(path, data);
}

export function exists(path: string): boolean {
  return nodeFs.existsSync(path);
}

export function readDir(path: string): FileInfo[] {
  return nodeFs.readdirSync(path).map((name: string) => {
    const full = nodePath.join(path, name);
    const stat = nodeFs.statSync(full);
    return {
      path: full,
      size: stat.size,
      isDir: stat.isDirectory(),
      isFile: stat.isFile(),
      modified: stat.mtime.toISOString(),
    };
  });
}

export function readDirRecursive(path: string): FileInfo[] {
  const entries: FileInfo[] = [];
  function walk(dir: string) {
    for (const name of nodeFs.readdirSync(dir)) {
      const full = nodePath.join(dir, name);
      const stat = nodeFs.statSync(full);
      entries.push({
        path: full,
        size: stat.size,
        isDir: stat.isDirectory(),
        isFile: stat.isFile(),
        modified: stat.mtime.toISOString(),
      });
      if (stat.isDirectory()) walk(full);
    }
  }
  walk(path);
  return entries;
}

export function ensureDir(path: string): void {
  nodeFs.mkdirSync(path, { recursive: true });
}

export function remove(path: string): void {
  if (nodeFs.existsSync(path)) {
    nodeFs.rmSync(path, { recursive: true, force: true });
  }
}

export function copy(src: string, dest: string): void {
  nodeFs.cpSync(src, dest, { recursive: true });
}

export function rename(oldPath: string, newPath: string): void {
  nodeFs.renameSync(oldPath, newPath);
}

export function stat(path: string): FileInfo {
  const s = nodeFs.statSync(path);
  return {
    path,
    size: s.size,
    isDir: s.isDirectory(),
    isFile: s.isFile(),
    modified: s.mtime.toISOString(),
  };
}

export function isDir(path: string): boolean {
  return nodeFs.existsSync(path) && nodeFs.statSync(path).isDirectory();
}

export function isFile(path: string): boolean {
  return nodeFs.existsSync(path) && nodeFs.statSync(path).isFile();
}

export function fileSize(path: string): number {
  return nodeFs.statSync(path).size;
}

export function cwd(): string {
  return process.cwd();
}

export function chdir(path: string): void {
  process.chdir(path);
}

export function listFiles(path: string): string[] {
  return nodeFs.readdirSync(path).filter((name: string) => {
    return nodeFs.statSync(nodePath.join(path, name)).isFile();
  });
}
