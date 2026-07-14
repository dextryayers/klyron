import { DockerConfig } from "./types.js";

export class DockerClient {
  private config: DockerConfig;

  constructor(config: DockerConfig) {
    this.config = config;
  }

  async execute(): Promise<void> {
    // TODO: implement
  }
}
