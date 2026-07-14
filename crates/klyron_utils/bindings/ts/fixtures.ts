// Test fixtures for klyron_utils
import { Klyron::UtilsConfig } from "./types.ts";

export const sampleConfig: Klyron::UtilsConfig = {
  enabled: true,
};

export const invalidConfig: Record<string, unknown> = {
  enabled: "not-a-boolean",
};
