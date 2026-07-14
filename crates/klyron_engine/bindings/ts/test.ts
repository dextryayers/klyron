// Tests for klyron_engine
import { assertEquals } from "https://deno.land/std/assert/mod.ts";
import { Klyron::EngineClient } from "./client.ts";
import { loadConfig } from "./config.ts";

Deno.test("klyron_engine client connect", async () => {
  const client = new Klyron::EngineClient("http://localhost:8080");
  const result = await client.connect();
  assertEquals(result.success, true);
});

Deno.test("klyron_engine load default config", () => {
  const config = loadConfig();
  assertEquals(config.enabled, true);
});
