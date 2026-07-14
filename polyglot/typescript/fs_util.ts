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
