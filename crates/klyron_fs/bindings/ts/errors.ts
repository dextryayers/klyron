export class FileSystemError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'FileSystemError';
  }
}

export class FileSystemHttpError extends FileSystemError {
  constructor(public status: number, message: string) {
    super(message);
    this.name = 'FileSystemHttpError';
  }
}
