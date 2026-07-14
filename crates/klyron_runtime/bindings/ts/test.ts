// Tests for klyron_runtime
import { assertEquals } from "https://deno.land/std/assert/mod.ts";
import { Klyron::RuntimeClient } from "./client.ts";
import { loadConfig } from "./config.ts";

Deno.test("klyron_runtime client connect", async () => {
  const client = new Klyron::RuntimeClient("http://localhost:8080");
  const result = await client.connect();
  assertEquals(result.success, true);
});

Deno.test("klyron_runtime load default config", () => {
  const config = loadConfig();
  assertEquals(config.enabled, true);
});
