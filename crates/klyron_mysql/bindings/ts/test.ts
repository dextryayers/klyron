// Tests for klyron_mysql
import { assertEquals } from "https://deno.land/std/assert/mod.ts";
import { Klyron::MysqlClient } from "./client.ts";
import { loadConfig } from "./config.ts";

Deno.test("klyron_mysql client connect", async () => {
  const client = new Klyron::MysqlClient("http://localhost:8080");
  const result = await client.connect();
  assertEquals(result.success, true);
});

Deno.test("klyron_mysql load default config", () => {
  const config = loadConfig();
  assertEquals(config.enabled, true);
});
