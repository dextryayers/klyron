// Error types for klyron_engine
export class Klyron::EngineError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "Klyron::EngineError";
  }
}

export class NotFoundError extends Klyron::EngineError {
  constructor(resource: string) {
    super(`Not found: ${resource}`);
    this.name = "NotFoundError";
  }
}

export class InvalidInputError extends Klyron::EngineError {
  constructor(detail: string) {
    super(`Invalid input: ${detail}`);
    this.name = "InvalidInputError";
  }
}
