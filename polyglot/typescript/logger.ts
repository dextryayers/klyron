import { LogLevel, LoggerConfig } from "./types";

const COLORS: Record<LogLevel, string> = {
  TRACE: "\x1b[90m",
  DEBUG: "\x1b[36m",
  INFO: "\x1b[32m",
  WARN: "\x1b[33m",
  ERROR: "\x1b[31m",
  FATAL: "\x1b[35m",
};
const RESET = "\x1b[0m";

const LEVEL_ORDER: Record<LogLevel, number> = {
  TRACE: 0, DEBUG: 1, INFO: 2, WARN: 3, ERROR: 4, FATAL: 5,
};

function timestamp(): string {
  return new Date().toISOString().replace("T", " ").slice(0, 23);
}

export class Logger {
  private config: LoggerConfig;

  constructor(config?: Partial<LoggerConfig>) {
    this.config = {
      minLevel: "INFO",
      jsonOutput: false,
      colorEnabled: true,
      ...config,
    };
  }

  private shouldLog(level: LogLevel): boolean {
    return LEVEL_ORDER[level] >= LEVEL_ORDER[this.config.minLevel];
  }

  private log(level: LogLevel, message: string, meta?: Record<string, unknown>): void {
    if (!this.shouldLog(level)) return;

    const entry = { timestamp: timestamp(), level, message, ...(meta ? { meta } : {}) };

    let output: string;
    if (this.config.jsonOutput) {
      output = JSON.stringify(entry);
    } else if (this.config.colorEnabled) {
      output = `${COLORS[level]}${timestamp()} [${level.padEnd(5)}] ${message}${RESET}`;
    } else {
      output = `${timestamp()} [${level.padEnd(5)}] ${message}`;
    }

    if (level === "ERROR" || level === "FATAL") {
      console.error(output);
    } else {
      console.log(output);
    }
  }

  trace(msg: string, meta?: Record<string, unknown>) { this.log("TRACE", msg, meta); }
  debug(msg: string, meta?: Record<string, unknown>) { this.log("DEBUG", msg, meta); }
  info(msg: string, meta?: Record<string, unknown>) { this.log("INFO", msg, meta); }
  warn(msg: string, meta?: Record<string, unknown>) { this.log("WARN", msg, meta); }
  error(msg: string, meta?: Record<string, unknown>) { this.log("ERROR", msg, meta); }
  fatal(msg: string, meta?: Record<string, unknown>) { this.log("FATAL", msg, meta); }
}

export const logger = new Logger();
