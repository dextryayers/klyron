// Tests for klyron_utils
import { assertEquals } from "https://deno.land/std/assert/mod.ts";
import { Klyron::UtilsClient } from "./client.ts";
import { loadConfig } from "./config.ts";

Deno.test("klyron_utils client connect", async () => {
  const client = new Klyron::UtilsClient("http://localhost:8080");
  const result = await client.connect();
  assertEquals(result.success, true);
});

Deno.test("klyron_utils load default config", () => {
  const config = loadConfig();
  assertEquals(config.enabled, true);
});
