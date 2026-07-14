// Klyron Runtime — WebSocket API (Web compatible)
// WebSocket, CloseEvent, MessageEvent

const CONNECTING = 0;
const OPEN = 1;
const CLOSING = 2;
const CLOSED = 3;

class WebSocket extends EventTarget {
  #url;
  #readyState = CONNECTING;
  #onopen = null;
  #onmessage = null;
  #onerror = null;
  #onclose = null;
  #readerTimer = null;
  #closed = false;

  constructor(url) {
    super();
    this.#url = url;
    this.#connect();
  }

  get url() { return this.#url; }
  get readyState() { return this.#readyState; }
  get protocol() { return ''; }
  get extensions() { return ''; }
  get bufferedAmount() { return 0; }
  get binaryType() { return 'blob'; }
  set binaryType(val) {}

  get onopen() { return this.#onopen; }
  set onopen(fn) {
    this.#onopen = fn;
    if (fn) this.addEventListener('open', fn);
  }
  get onmessage() { return this.#onmessage; }
  set onmessage(fn) {
    this.#onmessage = fn;
    if (fn) this.addEventListener('message', fn);
  }
  get onerror() { return this.#onerror; }
  set onerror(fn) {
    this.#onerror = fn;
    if (fn) this.addEventListener('error', fn);
  }
  get onclose() { return this.#onclose; }
  set onclose(fn) {
    this.#onclose = fn;
    if (fn) this.addEventListener('close', fn);
  }

  send(data) {
    if (this.#readyState !== OPEN) throw new Error('WebSocket is not open');
    if (typeof Klyron?.ws?.send === 'function') {
      Klyron.ws.send(String(data));
    }
  }

  close(code, reason) {
    if (this.#closed) return;
    this.#closed = true;
    this.#readyState = CLOSING;
    if (typeof Klyron?.ws?.close === 'function') {
      Klyron.ws.close(code || 1000, reason || '');
    }
    this.#cleanup();
    this.#readyState = CLOSED;
    this.#dispatchClose(1000, 'Normal closure', true);
  }

  #connect() {
    if (typeof Klyron?.ws?.connect === 'function') {
      try {
        Klyron.ws.connect(this.#url);
        this.#readyState = OPEN;
        this.#dispatchEvent(new Event('open'));
        this.#startReader();
      } catch (e) {
        this.#readyState = CLOSED;
        this.#dispatchEvent(new ErrorEvent('error', { error: e }));
        this.#dispatchClose(1006, e.message, false);
      }
    } else {
      queueMicrotask(() => {
        this.#readyState = OPEN;
        this.#dispatchEvent(new Event('open'));
        this.#startReader();
      });
    }
  }

  #startReader() {
    if (typeof Klyron?.ws?.receive !== 'function') return;
    this.#readerTimer = setInterval(() => {
      if (this.#closed || this.#readyState === CLOSED) {
        this.#cleanup();
        return;
      }
      try {
        const msg = Klyron.ws.receive();
        if (msg === null || msg === undefined) return;
        this.#dispatchEvent(new MessageEvent('message', { data: msg }));
      } catch (_) {
        this.#dispatchClose(1006, 'Read error', false);
        this.#cleanup();
      }
    }, 10);
  }

  #cleanup() {
    if (this.#readerTimer) {
      clearInterval(this.#readerTimer);
      this.#readerTimer = null;
    }
  }

  #dispatchClose(code, reason, wasClean) {
    this.#readyState = CLOSED;
    this.#dispatchEvent(new CloseEvent('close', { code, reason, wasClean }));
  }

  #dispatchEvent(event) {
    this.dispatchEvent(event);
  }

  static get CONNECTING() { return CONNECTING; }
  static get OPEN() { return OPEN; }
  static get CLOSING() { return CLOSING; }
  static get CLOSED() { return CLOSED; }
}

class CloseEvent extends Event {
  constructor(type, opts = {}) {
    super(type, opts);
    this.code = opts.code ?? 1005;
    this.reason = opts.reason ?? '';
    this.wasClean = opts.wasClean ?? true;
  }
}

class MessageEvent extends Event {
  constructor(type, opts = {}) {
    super(type, opts);
    this.data = opts.data ?? null;
    this.origin = opts.origin ?? '';
    this.lastEventId = opts.lastEventId ?? '';
    this.source = opts.source ?? null;
    this.ports = opts.ports ?? [];
  }
}

class ErrorEvent extends Event {
  constructor(type, opts = {}) {
    super(type, opts);
    this.message = opts.message ?? '';
    this.error = opts.error ?? null;
    this.filename = opts.filename ?? '';
    this.lineno = opts.lineno ?? 0;
    this.colno = opts.colno ?? 0;
  }
}

if (typeof globalThis.WebSocket !== 'function') {
  globalThis.WebSocket = WebSocket;
  globalThis.CloseEvent = CloseEvent;
  globalThis.MessageEvent = MessageEvent;
  globalThis.ErrorEvent = ErrorEvent;
}
