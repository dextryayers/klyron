export class DeployError extends Error {
  constructor(message: string) {
    super(`[Deploy] ${message}`);
    this.name = "DeployError";
  }
}

export class InitError extends DeployError {
  constructor(msg: string) { super(msg); }
}

export class OperationError extends DeployError {
  constructor(msg: string) { super(msg); }
}
