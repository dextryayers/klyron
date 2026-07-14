export class TelemetryError extends Error {
  constructor(message: string) {
    super(`[Telemetry] ${message}`);
    this.name = "TelemetryError";
  }
}

export class InitError extends TelemetryError {
  constructor(msg: string) { super(msg); }
}

export class OperationError extends TelemetryError {
  constructor(msg: string) { super(msg); }
}
