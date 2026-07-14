// Klyron Runtime — AbortController + AbortSignal (Web compatible)
// AbortController, AbortSignal

class AbortSignal extends EventTarget {
  constructor() {
    super();
    this._aborted = false;
    this._reason = undefined;
  }

  get aborted() { return this._aborted; }
  get reason() { return this._reason; }

  static abort(reason) {
    const signal = new AbortSignal();
    signal._aborted = true;
    signal._reason = reason || new DOMException('The operation was aborted', 'AbortError');
    return signal;
  }

  static timeout(ms) {
    const signal = new AbortSignal();
    setTimeout(() => {
      signal._aborted = true;
      signal._reason = new DOMException('The operation timed out', 'TimeoutError');
      signal.dispatchEvent(new Event('abort'));
    }, ms);
    return signal;
  }

  onabort() {
    this.addEventListener('abort', this._onabort);
  }
}

class AbortController {
  constructor() {
    this._signal = new AbortSignal();
    this._signal._controller = this;
  }

  get signal() { return this._signal; }

  abort(reason) {
    if (this._signal._aborted) return;
    this._signal._aborted = true;
    this._signal._reason = reason || new DOMException('The operation was aborted', 'AbortError');
    this._signal.dispatchEvent(new Event('abort'));
  }
}

if (typeof globalThis.AbortController !== 'function') {
  globalThis.AbortController = AbortController;
  globalThis.AbortSignal = AbortSignal;
}
