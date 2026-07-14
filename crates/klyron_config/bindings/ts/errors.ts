export class ConfigError extends Error {
  constructor(message: string) {
    super(`[Config] ${message}`);
    this.name = "ConfigError";
  }
}

export class InitError extends ConfigError {
  constructor(msg: string) { super(msg); }
}

export class OperationError extends ConfigError {
  constructor(msg: string) { super(msg); }
}
