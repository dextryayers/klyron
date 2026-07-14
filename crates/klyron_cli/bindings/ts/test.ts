// Tests for klyron_cli
import { assertEquals } from "https://deno.land/std/assert/mod.ts";
import { Klyron::CliClient } from "./client.ts";
import { loadConfig } from "./config.ts";

Deno.test("klyron_cli client connect", async () => {
  const client = new Klyron::CliClient("http://localhost:8080");
  const result = await client.connect();
  assertEquals(result.success, true);
});

Deno.test("klyron_cli load default config", () => {
  const config = loadConfig();
  assertEquals(config.enabled, true);
});
