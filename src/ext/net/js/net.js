import { op_net_connect, op_net_send, op_net_recv, op_net_close } from "ext:core/ops";

export async function connect(addr) { return op_net_connect(addr); }
export async function send(rid, data) { return op_net_send(rid, data); }
export async function recv(rid) { return op_net_recv(rid); }
export async function close(rid) { return op_net_close(rid); }

export default { connect, send, recv, close };
