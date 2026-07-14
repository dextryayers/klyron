import { DeployConfig } from "./types.js";

export class DeployClient {
  private config: DeployConfig;

  constructor(config: DeployConfig) {
    this.config = config;
  }

  async execute(): Promise<void> {
    // TODO: implement
  }
}
