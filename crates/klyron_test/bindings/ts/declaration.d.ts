declare module '@klyron/test' {
  export class TestClient {
    constructor(config?: Partial<{ enabled: boolean; verbose: boolean }>);
    version(): string;
  }
}
