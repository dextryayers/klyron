declare module "@klyron/telemetry" {
  export interface TelemetryConfig {
    version: string;
  }
  export class TelemetryClient {
    constructor(config: TelemetryConfig);
    execute(): Promise<void>;
  }
  export function loadConfig(): TelemetryConfig;
  export function version(): string;
}
