import { op_ffi_open, op_ffi_call } from "ext:core/ops";

export async function open(path) { return op_ffi_open(path); }
export async function call(libId, fnName, args) { return op_ffi_call(libId, fnName, JSON.stringify(args)); }

export default { open, call };
