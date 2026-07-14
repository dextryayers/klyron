// Test fixtures for klyron_sqlite
import { Klyron::SqliteConfig } from "./types.ts";

export const sampleConfig: Klyron::SqliteConfig = {
  enabled: true,
};

export const invalidConfig: Record<string, unknown> = {
  enabled: "not-a-boolean",
};
