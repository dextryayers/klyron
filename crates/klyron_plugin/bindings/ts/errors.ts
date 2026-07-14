export class PluginError extends Error {
  constructor(message: string) {
    super(`[Plugin] ${message}`);
    this.name = "PluginError";
  }
}

export class InitError extends PluginError {
  constructor(msg: string) { super(msg); }
}

export class OperationError extends PluginError {
  constructor(msg: string) { super(msg); }
}
