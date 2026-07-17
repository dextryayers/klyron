import { op_net_connect, op_net_send, op_net_recv, op_net_close, op_net_listen, op_net_accept, op_net_listen_close, op_net_send_bin, op_net_recv_bin, op_net_sockname, op_net_peername } from "ext:core/ops";

export async function connect(addr) { return op_net_connect(addr); }
export async function send(rid, data) { return op_net_send(rid, data); }
export async function sendBin(rid, data) { return op_net_send_bin(rid, data); }
export async function recv(rid) { return op_net_recv(rid); }
export async function recvBin(rid) { return op_net_recv_bin(rid); }
export async function close(rid) { return op_net_close(rid); }
export async function listen(addr) { return op_net_listen(addr); }
export async function accept(lrid) { return op_net_accept(lrid); }
export async function listenClose(lrid) { return op_net_listen_close(lrid); }
export async function sockname(rid) { return op_net_sockname(rid); }
export async function peername(rid) { return op_net_peername(rid); }

export default { connect, send, sendBin, recv, recvBin, close, listen, accept, listenClose, sockname, peername };
