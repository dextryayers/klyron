import { WorkspaceConfig } from "./types.js";

export class WorkspaceClient {
  private config: WorkspaceConfig;

  constructor(config: WorkspaceConfig) {
    this.config = config;
  }

  async execute(): Promise<void> {
  }
}
