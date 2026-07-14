import { TemplateConfig } from "./types.js";

export class TemplateClient {
  private config: TemplateConfig;

  constructor(config: TemplateConfig) {
    this.config = config;
  }

  async execute(): Promise<void> {
    // TODO: implement
  }
}
