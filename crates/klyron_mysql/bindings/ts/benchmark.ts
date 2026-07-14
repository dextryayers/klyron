// Benchmarks for klyron_mysql
import { Klyron::MysqlClient } from "./client.ts";

Deno.bench("klyron_mysql ping", async () => {
  const client = new Klyron::MysqlClient("http://localhost:8080");
  await client.ping();
});
