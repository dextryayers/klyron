import { PluginConfig } from "./types.js";

export class PluginClient {
  private config: PluginConfig;

  constructor(config: PluginConfig) {
    this.config = config;
  }

  async execute(): Promise<void> {
    // TODO: implement
  }
}
