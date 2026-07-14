declare module '@klyron/formatter' {
  export class FormatterClient {
    constructor(config?: Partial<{ enabled: boolean; verbose: boolean }>);
    version(): string;
  }
}
