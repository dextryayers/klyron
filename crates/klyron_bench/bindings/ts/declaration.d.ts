declare module '@klyron/bench' {
  export class BenchClient {
    constructor(config?: Partial<{ enabled: boolean; verbose: boolean }>);
    version(): string;
  }
}
