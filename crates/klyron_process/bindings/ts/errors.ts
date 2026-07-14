export class ProcessManagerError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'ProcessManagerError';
  }
}

export class ProcessManagerHttpError extends ProcessManagerError {
  constructor(public status: number, message: string) {
    super(message);
    this.name = 'ProcessManagerHttpError';
  }
}
