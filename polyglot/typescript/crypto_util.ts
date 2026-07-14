import { randomBytes, createHash } from "crypto";

export function sha256(data: string | Buffer): string {
  return createHash("sha256").update(data).digest("hex");
}

export function sha512(data: string | Buffer): string {
  return createHash("sha512").update(data).digest("hex");
}

export function md5(data: string | Buffer): string {
  return createHash("md5").update(data).digest("hex");
}

export function randomHex(bytes: number): string {
  return randomBytes(bytes).toString("hex");
}

export function uuidv4(): string {
  return randomBytes(16).toString("hex").replace(
    /(.{8})(.{4})(.{4})(.{4})(.{12})/,
    "$1-$2-4$3-8$4-$5"
  );
}

export function base64encode(data: string | Buffer): string {
  return Buffer.from(data).toString("base64");
}

export function base64decode(data: string): string {
  return Buffer.from(data, "base64").toString("utf-8");
}

export function hmacSha256(key: string, data: string): string {
  const { createHmac } = require("crypto");
  return createHmac("sha256", key).update(data).digest("hex");
}
