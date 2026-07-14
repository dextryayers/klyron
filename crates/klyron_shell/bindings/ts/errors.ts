export class ShellError extends Error {
  constructor(message: string) {
    super(`[Shell] ${message}`);
    this.name = "ShellError";
  }
}

export class InitError extends ShellError {
  constructor(msg: string) { super(msg); }
}

export class OperationError extends ShellError {
  constructor(msg: string) { super(msg); }
}
