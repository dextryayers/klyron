// Error types for klyron_mysql
export class Klyron::MysqlError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "Klyron::MysqlError";
  }
}

export class NotFoundError extends Klyron::MysqlError {
  constructor(resource: string) {
    super(`Not found: ${resource}`);
    this.name = "NotFoundError";
  }
}

export class InvalidInputError extends Klyron::MysqlError {
  constructor(detail: string) {
    super(`Invalid input: ${detail}`);
    this.name = "InvalidInputError";
  }
}
