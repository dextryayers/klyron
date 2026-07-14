declare module '@klyron/registry' {
  export class RegistryClient {
    constructor(config?: Partial<{ enabled: boolean; verbose: boolean }>);
    version(): string;
  }
}
