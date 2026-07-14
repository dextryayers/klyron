import { DnsRecord } from "./types";
import { promises as dns } from "dns";

export async function resolveA(hostname: string): Promise<string[]> {
  return dns.resolve4(hostname);
}

export async function resolveAAAA(hostname: string): Promise<string[]> {
  return dns.resolve6(hostname);
}

export async function resolveCname(hostname: string): Promise<string[]> {
  return dns.resolveCname(hostname);
}

export async function resolveMx(hostname: string): Promise<DnsRecord[]> {
  const records = await dns.resolveMx(hostname);
  return records.map((r) => ({
    name: hostname,
    recordType: "MX",
    value: `${r.exchange} pref=${r.priority}`,
    ttl: 0,
  }));
}

export async function resolveTxt(hostname: string): Promise<DnsRecord[]> {
  const records = await dns.resolveTxt(hostname);
  return records.map((r) => ({
    name: hostname,
    recordType: "TXT",
    value: r.join(""),
    ttl: 0,
  }));
}

export async function lookup(hostname: string): Promise<string[]> {
  const addresses = await dns.lookup(hostname, { all: true });
  return addresses.map((a) => a.address);
}
