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

export function deepMerge<T extends Record<string, unknown>>(target: T, source: Partial<T>): T {
  const result = { ...target };
  for (const [key, value] of Object.entries(source)) {
    if (value !== null && typeof value === "object" && !Array.isArray(value) &&
        result[key] !== null && typeof result[key] === "object" && !Array.isArray(result[key])) {
      result[key] = deepMerge(result[key] as Record<string, unknown>, value as Record<string, unknown>) as T[typeof key];
    } else if (value !== undefined) {
      result[key] = value as T[typeof key];
    }
  }
  return result;
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

export function getString(obj: Record<string, unknown>, key: string, defaultValue = ""): string {
  const val = obj[key];
  return typeof val === "string" ? val : defaultValue;
}

export function getNumber(obj: Record<string, unknown>, key: string, defaultValue = 0): number {
  const val = obj[key];
  return typeof val === "number" ? val : defaultValue;
}

export function getBool(obj: Record<string, unknown>, key: string, defaultValue = false): boolean {
  const val = obj[key];
  return typeof val === "boolean" ? val : defaultValue;
}

export function safeParse<T = JsonValue>(text: string, fallback: T | null = null): T | null {
  try { return JSON.parse(text) as T; }
  catch { return fallback; }
}
