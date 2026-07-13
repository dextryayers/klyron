import { op_html_escape, op_html_unescape, op_html_strip_tags } from "ext:core/ops";

export function escape(s) { return op_html_escape(s); }
export function unescape(s) { return op_html_unescape(s); }
export function stripTags(s) { return op_html_strip_tags(s); }

export default { escape, unescape, stripTags };
