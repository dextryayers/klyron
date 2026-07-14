// Benchmarks for klyron_ai
import { Klyron::AiClient } from "./client.ts";

Deno.bench("klyron_ai ping", async () => {
  const client = new Klyron::AiClient("http://localhost:8080");
  await client.ping();
});
