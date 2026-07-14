// Error types for klyron_ai
export class Klyron::AiError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "Klyron::AiError";
  }
}

export class NotFoundError extends Klyron::AiError {
  constructor(resource: string) {
    super(`Not found: ${resource}`);
    this.name = "NotFoundError";
  }
}

export class InvalidInputError extends Klyron::AiError {
  constructor(detail: string) {
    super(`Invalid input: ${detail}`);
    this.name = "InvalidInputError";
  }
}
