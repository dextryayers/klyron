// Benchmarks for klyron_cli
import { Klyron::CliClient } from "./client.ts";

Deno.bench("klyron_cli ping", async () => {
  const client = new Klyron::CliClient("http://localhost:8080");
  await client.ping();
});
