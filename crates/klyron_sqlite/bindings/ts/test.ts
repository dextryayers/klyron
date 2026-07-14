// Tests for klyron_sqlite
import { assertEquals } from "https://deno.land/std/assert/mod.ts";
import { Klyron::SqliteClient } from "./client.ts";
import { loadConfig } from "./config.ts";

Deno.test("klyron_sqlite client connect", async () => {
  const client = new Klyron::SqliteClient("http://localhost:8080");
  const result = await client.connect();
  assertEquals(result.success, true);
});

Deno.test("klyron_sqlite load default config", () => {
  const config = loadConfig();
  assertEquals(config.enabled, true);
});
