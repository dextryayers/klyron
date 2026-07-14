import { ProcessResult, ExecOptions } from "./types";

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

export async function execShell(command: string, options?: ExecOptions): Promise<ProcessResult> {
  const { exec } = await import("child_process");
  return new Promise((resolve) => {
    const child = exec(command, { cwd: options?.cwd, timeout: options?.timeout }, (error, stdout, stderr) => {
      resolve({
        stdout,
        stderr,
        exitCode: error?.code ?? 0,
        success: !error,
      });
    });
  });
}

export async function spawnDetached(command: string, args: string[] = []): Promise<number> {
  const { spawn } = await import("child_process");
  const child = spawn(command, args, {
    detached: true,
    stdio: "ignore",
  });
  child.unref();
  return child.pid || -1;
}

export function kill(pid: number, signal: NodeJS.Signals = "SIGTERM"): boolean {
  try {
    process.kill(pid, signal);
    return true;
  } catch {
    return false;
  }
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

export function setEnv(key: string, value: string): void {
  process.env[key] = value;
}

export function pid(): number {
  return process.pid;
}

export function cwd(): string {
  return process.cwd();
}

export function chdir(dir: string): void {
  process.chdir(dir);
}

export function exit(code = 0): never {
  process.exit(code);
}

export function uptime(): number {
  return process.uptime();
}

export function memoryUsage(): { rss: number; heapUsed: number; heapTotal: number } {
  const usage = process.memoryUsage();
  return { rss: usage.rss, heapUsed: usage.heapUsed, heapTotal: usage.heapTotal };
}
