import { op_web_fetch, op_web_text_encode, op_web_text_decode, op_web_base64_encode, op_web_base64_decode } from "ext:core/ops";

export async function fetch(url) { return op_web_fetch(url); }
export function textEncode(s) { return op_web_text_encode(s); }
export function textDecode(bytes) { return op_web_text_decode(bytes); }
export function base64Encode(data) { return op_web_base64_encode(data); }
export function base64Decode(s) { return op_web_base64_decode(s); }

export default { fetch, textEncode, textDecode, base64Encode, base64Decode };
