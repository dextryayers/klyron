import { op_ws_connect, op_ws_send, op_ws_recv, op_ws_close } from "ext:core/ops";

const CONNECTING = 0;
const OPEN = 1;
const CLOSING = 2;
const CLOSED = 3;

let nativeWsId = 0;

class WebSocket {
  #url;
  #readyState = CONNECTING;
  #id = -1;
  #onopen = null;
  #onmessage = null;
  #onerror = null;
  #onclose = null;
  #readerTimer = null;
  #closed = false;

  constructor(url) {
    this.#url = url;
    try {
      const result = op_ws_connect(url);
      this.#id = result.id;
    } catch (e) {
      this.#readyState = CLOSED;
      queueMicrotask(() => {
        if (this.#onerror) this.#onerror(new ErrorEvent("error", { error: e }));
        if (this.#onclose) this.#onclose(new CloseEvent("close", { code: 1006, reason: "Connection failed" }));
      });
      return;
    }
    this.#startReader();
  }

  get url() { return this.#url; }
  get readyState() { return this.#readyState; }

  set onopen(fn) { this.#onopen = fn; }
  set onmessage(fn) { this.#onmessage = fn; }
  set onerror(fn) { this.#onerror = fn; }
  set onclose(fn) { this.#onclose = fn; }

  send(data) {
    if (this.#readyState !== OPEN) throw new Error("WebSocket is not open");
    op_ws_send(this.#id, String(data));
  }

  close() {
    if (this.#closed) return;
    this.#closed = true;
    this.#readyState = CLOSING;
    try {
      op_ws_close(this.#id);
    } catch (_) {}
    if (this.#readerTimer) {
      clearInterval(this.#readerTimer);
      this.#readerTimer = null;
    }
    this.#readyState = CLOSED;
    if (this.#onclose) {
      this.#onclose(new CloseEvent("close", { code: 1000, reason: "Normal closure" }));
    }
  }

  #startReader() {
    this.#readerTimer = setInterval(() => {
      if (this.#closed || this.#readyState === CLOSED) {
        if (this.#readerTimer) {
          clearInterval(this.#readerTimer);
          this.#readerTimer = null;
        }
        return;
      }
      try {
        const msg = op_ws_recv(this.#id);
        if (msg === "__no_message__") return;
        if (msg === "__connected__") {
          this.#readyState = OPEN;
          if (this.#onopen) this.#onopen(new Event("open"));
          return;
        }
        if (msg === "__close__") {
          this.close();
          return;
        }
        if (msg?.startsWith("__error__:")) {
          const errMsg = msg.slice(10);
          this.#readyState = CLOSED;
          if (this.#onerror) this.#onerror(new ErrorEvent("error", { error: new Error(errMsg) }));
          if (this.#onclose) this.#onclose(new CloseEvent("close", { code: 1006, reason: errMsg }));
          if (this.#readerTimer) {
            clearInterval(this.#readerTimer);
            this.#readerTimer = null;
          }
          return;
        }
        if (msg?.startsWith("__text__:")) {
          const text = msg.slice(9);
          if (this.#onmessage) this.#onmessage(new MessageEvent("message", { data: text }));
          return;
        }
        if (msg?.startsWith("__binary__:")) {
          const data = msg.slice(11);
          if (this.#onmessage) this.#onmessage(new MessageEvent("message", { data }));
          return;
        }
      } catch (_) {
        this.close();
      }
    }, 10);
  }
}

class Event {
  constructor(type, opts = {}) {
    this.type = type;
    this.bubbles = opts.bubbles ?? false;
    this.cancelable = opts.cancelable ?? false;
  }
}

class ErrorEvent extends Event {
  constructor(type, opts = {}) {
    super(type, opts);
    this.message = opts.message ?? "";
    this.error = opts.error ?? null;
  }
}

class CloseEvent extends Event {
  constructor(type, opts = {}) {
    super(type, opts);
    this.code = opts.code ?? 1005;
    this.reason = opts.reason ?? "";
    this.wasClean = opts.wasClean ?? true;
  }
}

class MessageEvent extends Event {
  constructor(type, opts = {}) {
    super(type, opts);
    this.data = opts.data ?? null;
    this.origin = opts.origin ?? "";
  }
}

export { WebSocket, Event, MessageEvent, CloseEvent, ErrorEvent };
