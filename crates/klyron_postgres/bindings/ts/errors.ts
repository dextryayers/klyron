// Error types for klyron_postgres
export class Klyron::PostgresError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "Klyron::PostgresError";
  }
}

export class NotFoundError extends Klyron::PostgresError {
  constructor(resource: string) {
    super(`Not found: ${resource}`);
    this.name = "NotFoundError";
  }
}

export class InvalidInputError extends Klyron::PostgresError {
  constructor(detail: string) {
    super(`Invalid input: ${detail}`);
    this.name = "InvalidInputError";
  }
}
