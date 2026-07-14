// Test fixtures for klyron_runtime
import { Klyron::RuntimeConfig } from "./types.ts";

export const sampleConfig: Klyron::RuntimeConfig = {
  enabled: true,
};

export const invalidConfig: Record<string, unknown> = {
  enabled: "not-a-boolean",
};
