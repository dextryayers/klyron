export class StringDecoder {
  #encoding;

  constructor(encoding = "utf8") {
    this.#encoding = encoding;
  }

  write(buffer) {
    if (typeof buffer === "string") return buffer;
    return new TextDecoder(this.#encoding).decode(buffer);
  }

  end(buffer) {
    return this.write(buffer || new Uint8Array(0));
  }

  text(buffer, start, end) {
    return new TextDecoder(this.#encoding).decode(buffer.subarray(start, end));
  }

  fillLast(buffer) {
    if (!buffer || buffer.length === 0) return "";
    return this.write(buffer);
  }

  get encoding() {
    return this.#encoding;
  }
}

export default { StringDecoder };
