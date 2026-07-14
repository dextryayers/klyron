// Test fixtures for klyron_updater
import { Klyron::UpdaterConfig } from "./types.ts";

export const sampleConfig: Klyron::UpdaterConfig = {
  enabled: true,
};

export const invalidConfig: Record<string, unknown> = {
  enabled: "not-a-boolean",
};
