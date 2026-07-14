export class DockerError extends Error {
  constructor(message: string) {
    super(`[Docker] ${message}`);
    this.name = "DockerError";
  }
}

export class InitError extends DockerError {
  constructor(msg: string) { super(msg); }
}

export class OperationError extends DockerError {
  constructor(msg: string) { super(msg); }
}
