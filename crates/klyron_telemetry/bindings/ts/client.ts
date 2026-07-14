import { TelemetryConfig } from "./types.js";

export class TelemetryClient {
  private config: TelemetryConfig;

  constructor(config: TelemetryConfig) {
    this.config = config;
  }

  async execute(): Promise<void> {
    // TODO: implement
  }
}
