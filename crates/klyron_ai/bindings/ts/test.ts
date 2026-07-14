// Tests for klyron_ai
import { assertEquals } from "https://deno.land/std/assert/mod.ts";
import { Klyron::AiClient } from "./client.ts";
import { loadConfig } from "./config.ts";

Deno.test("klyron_ai client connect", async () => {
  const client = new Klyron::AiClient("http://localhost:8080");
  const result = await client.connect();
  assertEquals(result.success, true);
});

Deno.test("klyron_ai load default config", () => {
  const config = loadConfig();
  assertEquals(config.enabled, true);
});
