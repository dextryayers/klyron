// Error types for klyron_runtime
export class Klyron::RuntimeError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "Klyron::RuntimeError";
  }
}

export class NotFoundError extends Klyron::RuntimeError {
  constructor(resource: string) {
    super(`Not found: ${resource}`);
    this.name = "NotFoundError";
  }
}

export class InvalidInputError extends Klyron::RuntimeError {
  constructor(detail: string) {
    super(`Invalid input: ${detail}`);
    this.name = "InvalidInputError";
  }
}
