import { ShellConfig } from "./types.js";

export class ShellClient {
  private config: ShellConfig;

  constructor(config: ShellConfig) {
    this.config = config;
  }

  async execute(): Promise<void> {
  }
}
