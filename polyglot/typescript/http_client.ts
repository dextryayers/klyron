import { HttpResponse } from "./types";

export async function fetch(url: string, options: RequestInit = {}): Promise<HttpResponse> {
  const response = await globalThis.fetch(url, options);
  const headers: Record<string, string> = {};
  response.headers.forEach((v, k) => { headers[k] = v; });
  return {
    status: response.status,
    statusText: response.statusText,
    headers,
    body: await response.text(),
    ok: response.ok,
  };
}

export async function get(url: string): Promise<HttpResponse> {
  return fetch(url);
}

export async function post(url: string, body: unknown, contentType = "application/json"): Promise<HttpResponse> {
  return fetch(url, {
    method: "POST",
    headers: { "Content-Type": contentType },
    body: typeof body === "string" ? body : JSON.stringify(body),
  });
}

export async function put(url: string, body: unknown, contentType = "application/json"): Promise<HttpResponse> {
  return fetch(url, {
    method: "PUT",
    headers: { "Content-Type": contentType },
    body: typeof body === "string" ? body : JSON.stringify(body),
  });
}

export async function del(url: string): Promise<HttpResponse> {
  return fetch(url, { method: "DELETE" });
}

export async function patch(url: string, body: unknown, contentType = "application/json"): Promise<HttpResponse> {
  return fetch(url, {
    method: "PATCH",
    headers: { "Content-Type": contentType },
    body: typeof body === "string" ? body : JSON.stringify(body),
  });
}

export async function head(url: string): Promise<HttpResponse> {
  return fetch(url, { method: "HEAD" });
}

export async function getJson<T>(url: string): Promise<T> {
  const res = await get(url);
  return JSON.parse(res.body);
}

export async function postJson<T>(url: string, data: unknown): Promise<T> {
  const res = await post(url, data);
  return JSON.parse(res.body);
}

export async function putJson<T>(url: string, data: unknown): Promise<T> {
  const res = await put(url, data);
  return JSON.parse(res.body);
}
