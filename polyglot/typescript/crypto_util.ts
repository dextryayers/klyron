import { randomBytes, createHash, createHmac } from "crypto";

export function sha256(data: string | Buffer): string {
  return createHash("sha256").update(data).digest("hex");
}

export function sha512(data: string | Buffer): string {
  return createHash("sha512").update(data).digest("hex");
}

export function md5(data: string | Buffer): string {
  return createHash("md5").update(data).digest("hex");
}

export function sha1(data: string | Buffer): string {
  return createHash("sha1").update(data).digest("hex");
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
  return createHmac("sha256", key).update(data).digest("hex");
}

export function hmacSha512(key: string, data: string): string {
  return createHmac("sha512", key).update(data).digest("hex");
}

export function hexEncode(data: Buffer): string {
  return data.toString("hex");
}

export function hexDecode(hex: string): Buffer {
  return Buffer.from(hex, "hex");
}

export function randomInt(min: number, max: number): number {
  const range = max - min + 1;
  const bytes = Math.ceil(Math.log2(range) / 8);
  const buf = randomBytes(bytes);
  let val = 0;
  for (let i = 0; i < bytes; i++) {
    val = (val << 8) | buf[i];
  }
  return min + (val % range);
}

export function pbkdf2Sync(password: string, salt: string, iterations: number, keylen: number): string {
  const { pbkdf2Sync } = require("crypto");
  return pbkdf2Sync(password, salt, iterations, keylen, "sha256").toString("hex");
}
