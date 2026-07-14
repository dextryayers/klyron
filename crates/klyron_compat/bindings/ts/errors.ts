export class CompatError extends Error {
  constructor(message: string) {
    super(`[Compat] ${message}`);
    this.name = "CompatError";
  }
}

export class InitError extends CompatError {
  constructor(msg: string) { super(msg); }
}

export class OperationError extends CompatError {
  constructor(msg: string) { super(msg); }
}
