// Test fixtures for klyron_mysql
import { Klyron::MysqlConfig } from "./types.ts";

export const sampleConfig: Klyron::MysqlConfig = {
  enabled: true,
};

export const invalidConfig: Record<string, unknown> = {
  enabled: "not-a-boolean",
};
