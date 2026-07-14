declare module '@klyron/transpiler' {
  export class TranspilerClient {
    constructor(config?: Partial<{ enabled: boolean; verbose: boolean }>);
    version(): string;
  }
}
