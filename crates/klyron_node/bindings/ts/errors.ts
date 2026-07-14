export class NodeGlobalsError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'NodeGlobalsError';
  }
}

export class NodeGlobalsHttpError extends NodeGlobalsError {
  constructor(public status: number, message: string) {
    super(message);
    this.name = 'NodeGlobalsHttpError';
  }
}
