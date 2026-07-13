import { op_http_serve, op_http_stop } from "ext:core/ops";

export async function serve(addr) { return op_http_serve(addr); }
export async function stop() { return op_http_stop(); }

export default { serve, stop };
