// Benchmarks for klyron_utils
import { Klyron::UtilsClient } from "./client.ts";

Deno.bench("klyron_utils ping", async () => {
  const client = new Klyron::UtilsClient("http://localhost:8080");
  await client.ping();
});
