// Benchmarks for klyron_updater
import { Klyron::UpdaterClient } from "./client.ts";

Deno.bench("klyron_updater ping", async () => {
  const client = new Klyron::UpdaterClient("http://localhost:8080");
  await client.ping();
});
