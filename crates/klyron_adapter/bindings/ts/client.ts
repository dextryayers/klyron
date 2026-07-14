import { AdapterConfig } from "./types.js";

export class AdapterClient {
  private config: AdapterConfig;

  constructor(config: AdapterConfig) {
    this.config = config;
  }

  async execute(): Promise<void> {
  }
}
