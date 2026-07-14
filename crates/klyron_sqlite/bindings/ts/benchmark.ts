// Benchmarks for klyron_sqlite
import { Klyron::SqliteClient } from "./client.ts";

Deno.bench("klyron_sqlite ping", async () => {
  const client = new Klyron::SqliteClient("http://localhost:8080");
  await client.ping();
});
