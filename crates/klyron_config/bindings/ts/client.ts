import { ConfigConfig } from "./types.js";

export class ConfigClient {
  private config: ConfigConfig;

  constructor(config: ConfigConfig) {
    this.config = config;
  }

  async execute(): Promise<void> {
    // TODO: implement
  }
}
