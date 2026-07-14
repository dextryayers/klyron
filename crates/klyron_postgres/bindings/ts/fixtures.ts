// Test fixtures for klyron_postgres
import { Klyron::PostgresConfig } from "./types.ts";

export const sampleConfig: Klyron::PostgresConfig = {
  enabled: true,
};

export const invalidConfig: Record<string, unknown> = {
  enabled: "not-a-boolean",
};
