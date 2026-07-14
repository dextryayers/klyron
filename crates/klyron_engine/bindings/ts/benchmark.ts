// Benchmarks for klyron_engine
import { Klyron::EngineClient } from "./client.ts";

Deno.bench("klyron_engine ping", async () => {
  const client = new Klyron::EngineClient("http://localhost:8080");
  await client.ping();
});
