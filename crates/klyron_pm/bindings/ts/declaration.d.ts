declare module '@klyron/pm' {
  export class PmClient {
    constructor(config?: Partial<{ enabled: boolean; verbose: boolean }>);
    version(): string;
  }
}
