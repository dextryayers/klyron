declare module '@klyron/linter' {
  export class LinterClient {
    constructor(config?: Partial<{ enabled: boolean; verbose: boolean }>);
    version(): string;
  }
}
