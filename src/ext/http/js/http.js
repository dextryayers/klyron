import { op_http_serve, op_http_stop, op_http_json, op_http_html, op_http_text, op_http_redirect } from "ext:core/ops";

export async function serve(addr) { return op_http_serve(addr); }
export async function stop() { return op_http_stop(); }

export function json(data, status = 200) {
  return op_http_json(typeof data === 'string' ? data : JSON.stringify(data), status);
}

export function html(data, status = 200) {
  return op_http_html(String(data), status);
}

export function text(data, status = 200) {
  return op_http_text(String(data), status);
}

export function redirect(location, status = 302) {
  return op_http_redirect(location, status);
}

export default { serve, stop, json, html, text, redirect };
