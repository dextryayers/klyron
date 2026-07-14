// Error types for klyron_utils
export class Klyron::UtilsError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "Klyron::UtilsError";
  }
}

export class NotFoundError extends Klyron::UtilsError {
  constructor(resource: string) {
    super(`Not found: ${resource}`);
    this.name = "NotFoundError";
  }
}

export class InvalidInputError extends Klyron::UtilsError {
  constructor(detail: string) {
    super(`Invalid input: ${detail}`);
    this.name = "InvalidInputError";
  }
}
