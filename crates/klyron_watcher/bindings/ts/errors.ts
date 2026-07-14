export class WatcherError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'WatcherError';
  }
}
