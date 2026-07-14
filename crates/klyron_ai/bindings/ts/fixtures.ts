// Test fixtures for klyron_ai
import { Klyron::AiConfig } from "./types.ts";

export const sampleConfig: Klyron::AiConfig = {
  enabled: true,
};

export const invalidConfig: Record<string, unknown> = {
  enabled: "not-a-boolean",
};
