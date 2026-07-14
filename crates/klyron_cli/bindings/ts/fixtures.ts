// Test fixtures for klyron_cli
import { Klyron::CliConfig } from "./types.ts";

export const sampleConfig: Klyron::CliConfig = {
  enabled: true,
};

export const invalidConfig: Record<string, unknown> = {
  enabled: "not-a-boolean",
};
