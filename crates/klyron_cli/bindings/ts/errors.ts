// Error types for klyron_cli
export class Klyron::CliError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "Klyron::CliError";
  }
}

export class NotFoundError extends Klyron::CliError {
  constructor(resource: string) {
    super(`Not found: ${resource}`);
    this.name = "NotFoundError";
  }
}

export class InvalidInputError extends Klyron::CliError {
  constructor(detail: string) {
    super(`Invalid input: ${detail}`);
    this.name = "InvalidInputError";
  }
}
