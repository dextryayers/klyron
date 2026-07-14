export class WorkspaceError extends Error {
  constructor(message: string) {
    super(`[Workspace] ${message}`);
    this.name = "WorkspaceError";
  }
}

export class InitError extends WorkspaceError {
  constructor(msg: string) { super(msg); }
}

export class OperationError extends WorkspaceError {
  constructor(msg: string) { super(msg); }
}
