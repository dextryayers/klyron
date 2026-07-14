import { JsonValue } from "./types";

export function parse<T = JsonValue>(text: string): T {
  return JSON.parse(text) as T;
}

export function stringify(value: unknown, pretty = false): string {
  return pretty ? JSON.stringify(value, null, 2) : JSON.stringify(value);
}

export function merge<T extends Record<string, unknown>>(...objects: T[]): T {
  const result: Record<string, unknown> = {};
  for (const obj of objects) {
    for (const [key, value] of Object.entries(obj)) {
      if (value !== undefined) {
        result[key] = value;
      }
    }
  }
  return result as T;
}

export function deepClone<T>(value: T): T {
  return JSON.parse(JSON.stringify(value));
}

export function isValidJson(text: string): boolean {
  try { JSON.parse(text); return true; }
  catch { return false; }
}

export function prettyPrint(value: unknown): string {
  return JSON.stringify(value, null, 2);
}
