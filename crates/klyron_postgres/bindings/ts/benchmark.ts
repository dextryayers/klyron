// Benchmarks for klyron_postgres
import { Klyron::PostgresClient } from "./client.ts";

Deno.bench("klyron_postgres ping", async () => {
  const client = new Klyron::PostgresClient("http://localhost:8080");
  await client.ping();
});
