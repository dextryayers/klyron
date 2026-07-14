import { CompatConfig } from "./types.js";

export class CompatClient {
  private config: CompatConfig;

  constructor(config: CompatConfig) {
    this.config = config;
  }

  async execute(): Promise<void> {
  }
}
