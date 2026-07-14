declare module '@klyron/bundler' {
  export class BundlerClient {
    constructor(config?: Partial<{ enabled: boolean; verbose: boolean }>);
    version(): string;
  }
}
