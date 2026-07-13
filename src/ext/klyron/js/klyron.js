import { op_klyron_version, op_klyron_arch, op_klyron_platform } from "ext:core/ops";

export function version() { return op_klyron_version(); }
export function arch() { return op_klyron_arch(); }
export function platform() { return op_klyron_platform(); }

export default { version, arch, platform };
