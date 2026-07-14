// Error types for klyron_updater
export class Klyron::UpdaterError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "Klyron::UpdaterError";
  }
}

export class NotFoundError extends Klyron::UpdaterError {
  constructor(resource: string) {
    super(`Not found: ${resource}`);
    this.name = "NotFoundError";
  }
}

export class InvalidInputError extends Klyron::UpdaterError {
  constructor(detail: string) {
    super(`Invalid input: ${detail}`);
    this.name = "InvalidInputError";
  }
}
