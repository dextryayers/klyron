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

export async function resolveNs(hostname: string): Promise<string[]> {
  return dns.resolveNs(hostname);
}

export async function resolveSrv(hostname: string): Promise<DnsRecord[]> {
  const records = await dns.resolveSrv(hostname);
  return records.map((r) => ({
    name: hostname,
    recordType: "SRV",
    value: `${r.name}:${r.port} priority=${r.priority} weight=${r.weight}`,
    ttl: 0,
  }));
}

export async function lookup(hostname: string): Promise<string[]> {
  const addresses = await dns.lookup(hostname, { all: true });
  return addresses.map((a) => a.address);
}

export async function reverseLookup(ip: string): Promise<string> {
  const hostnames = await dns.reverse(ip);
  return hostnames[0] || "";
}

export async function isReachable(host: string, port: number, timeoutMs = 3000): Promise<boolean> {
  const net = await import("net");
  return new Promise((resolve) => {
    const socket = new net.Socket();
    socket.setTimeout(timeoutMs);
    socket.on("connect", () => { socket.destroy(); resolve(true); });
    socket.on("error", () => resolve(false));
    socket.on("timeout", () => { socket.destroy(); resolve(false); });
    socket.connect(port, host);
  });
}
