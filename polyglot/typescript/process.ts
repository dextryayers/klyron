import { ProcessResult } from "./types";
import { fs } from "./fs_util";

export async function exec(command: string, args: string[] = []): Promise<ProcessResult> {
  const { spawn } = await import("child_process");
  return new Promise((resolve, reject) => {
    const child = spawn(command, args);
    let stdout = "";
    let stderr = "";
    child.stdout.on("data", (d: Buffer) => { stdout += d.toString(); });
    child.stderr.on("data", (d: Buffer) => { stderr += d.toString(); });
    child.on("close", (code: number | null) => {
      resolve({ stdout, stderr, exitCode: code, success: code === 0 });
    });
    child.on("error", reject);
  });
}

export function which(program: string): string | null {
  try {
    const result = require("child_process").execSync(`which ${program}`, { encoding: "utf8" });
    return result.trim() || null;
  } catch { return null; }
}

export function sleep(ms: number): Promise<void> {
  return new Promise((r) => setTimeout(r, ms));
}

export function env(key: string): string | undefined {
  return process.env[key];
}

export function pid(): number {
  return process.pid;
}
