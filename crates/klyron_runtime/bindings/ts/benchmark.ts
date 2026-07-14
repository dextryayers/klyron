// Benchmarks for klyron_runtime
import { Klyron::RuntimeClient } from "./client.ts";

Deno.bench("klyron_runtime ping", async () => {
  const client = new Klyron::RuntimeClient("http://localhost:8080");
  await client.ping();
});
