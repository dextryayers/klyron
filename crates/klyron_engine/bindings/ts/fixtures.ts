// Test fixtures for klyron_engine
import { Klyron::EngineConfig } from "./types.ts";

export const sampleConfig: Klyron::EngineConfig = {
  enabled: true,
};

export const invalidConfig: Record<string, unknown> = {
  enabled: "not-a-boolean",
};
