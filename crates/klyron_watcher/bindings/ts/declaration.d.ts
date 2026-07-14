declare module '@klyron/watcher' {
  export class WatcherClient {
    constructor(config?: Partial<{ enabled: boolean; verbose: boolean }>);
    version(): string;
  }
}
