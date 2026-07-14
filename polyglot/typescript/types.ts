export type JsonValue = string | number | boolean | null | JsonValue[] | { [key: string]: JsonValue };

export interface ProcessResult {
  stdout: string;
  stderr: string;
  exitCode: number | null;
  success: boolean;
}

export interface FileInfo {
  path: string;
  size: number;
  isDir: boolean;
  isFile: boolean;
  modified: string | null;
}

export interface HttpResponse {
  status: number;
  statusText: string;
  headers: Record<string, string>;
  body: string;
  ok: boolean;
}

export interface DnsRecord {
  name: string;
  recordType: string;
  value: string;
  ttl: number;
}

export interface CacheEntry<T> {
  value: T;
  expiresAt: number | null;
  tags: string[];
}

export type LogLevel = "TRACE" | "DEBUG" | "INFO" | "WARN" | "ERROR" | "FATAL";

export interface LoggerConfig {
  minLevel: LogLevel;
  jsonOutput: boolean;
  filePath?: string;
  colorEnabled: boolean;
}
