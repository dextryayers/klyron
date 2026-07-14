export class TemplateError extends Error {
  constructor(message: string) {
    super(`[Template] ${message}`);
    this.name = "TemplateError";
  }
}

export class InitError extends TemplateError {
  constructor(msg: string) { super(msg); }
}

export class OperationError extends TemplateError {
  constructor(msg: string) { super(msg); }
}
