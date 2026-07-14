// Error types for klyron_sqlite
export class Klyron::SqliteError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "Klyron::SqliteError";
  }
}

export class NotFoundError extends Klyron::SqliteError {
  constructor(resource: string) {
    super(`Not found: ${resource}`);
    this.name = "NotFoundError";
  }
}

export class InvalidInputError extends Klyron::SqliteError {
  constructor(detail: string) {
    super(`Invalid input: ${detail}`);
    this.name = "InvalidInputError";
  }
}
