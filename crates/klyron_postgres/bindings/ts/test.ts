// Tests for klyron_postgres
import { assertEquals } from "https://deno.land/std/assert/mod.ts";
import { Klyron::PostgresClient } from "./client.ts";
import { loadConfig } from "./config.ts";

Deno.test("klyron_postgres client connect", async () => {
  const client = new Klyron::PostgresClient("http://localhost:8080");
  const result = await client.connect();
  assertEquals(result.success, true);
});

Deno.test("klyron_postgres load default config", () => {
  const config = loadConfig();
  assertEquals(config.enabled, true);
});
