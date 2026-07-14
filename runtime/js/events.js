// Klyron Runtime — EventTarget + Event (Web compatible)
// EventTarget, Event, CustomEvent

class Event {
  constructor(type, opts = {}) {
    this._type = type;
    this._bubbles = opts.bubbles || false;
    this._cancelable = opts.cancelable || false;
    this._composed = opts.composed || false;
    this._defaultPrevented = false;
    this._propagationStopped = false;
    this._immediatePropagationStopped = false;
    this._target = null;
    this._currentTarget = null;
    this._timeStamp = Date.now();
  }
  get type() { return this._type; }
  get target() { return this._target; }
  get currentTarget() { return this._currentTarget; }
  get bubbles() { return this._bubbles; }
  get cancelable() { return this._cancelable; }
  get defaultPrevented() { return this._defaultPrevented; }
  get composed() { return this._composed; }
  get timeStamp() { return this._timeStamp; }
  get eventPhase() { return 0; }
  get srcElement() { return this._target; }
  get returnValue() { return !this._defaultPrevented; }
  get cancelBubble() { return this._propagationStopped; }
  set cancelBubble(v) { if (v) this._propagationStopped = true; }

  composedPath() { return []; }
  preventDefault() { if (this._cancelable) this._defaultPrevented = true; }
  stopPropagation() { this._propagationStopped = true; }
  stopImmediatePropagation() { this._immediatePropagationStopped = true; }

  initEvent(type, bubbles, cancelable) {
    this._type = type;
    this._bubbles = !!bubbles;
    this._cancelable = !!cancelable;
  }
}

class CustomEvent extends Event {
  constructor(type, opts = {}) {
    super(type, opts);
    this._detail = opts.detail || null;
  }
  get detail() { return this._detail; }
}

class EventTarget {
  constructor() {
    this._listeners = new Map();
  }

  addEventListener(type, callback, options) {
    if (!callback) return;
    const capture = typeof options === 'object' ? !!options.capture : !!options;
    const once = typeof options === 'object' ? !!options.once : false;
    const passive = typeof options === 'object' ? !!options.passive : false;
    const key = type + ':' + capture;
    if (!this._listeners.has(key)) this._listeners.set(key, []);
    this._listeners.get(key).push({ callback, once, passive, signal: options?.signal || null });
  }

  removeEventListener(type, callback, options) {
    const capture = typeof options === 'object' ? !!options.capture : !!options;
    const key = type + ':' + capture;
    const listeners = this._listeners.get(key);
    if (!listeners) return;
    this._listeners.set(key, listeners.filter(l => l.callback !== callback));
  }

  dispatchEvent(event) {
    event._target = this;
    event._currentTarget = this;
    const key = event.type + ':false';
    const listeners = [...(this._listeners.get(key) || [])];
    for (const l of listeners) {
      if (event._immediatePropagationStopped) break;
      try {
        if (typeof l.callback === 'function') l.callback.call(this, event);
        else if (typeof l.callback.handleEvent === 'function') l.callback.handleEvent(event);
      } catch (e) {
        queueMicrotask(() => { throw e; });
      }
      if (l.once) this.removeEventListener(event.type, l.callback, false);
      if (l.signal) {
        const ab = l.signal;
        if (ab.aborted) this.removeEventListener(event.type, l.callback, false);
      }
    }
    return !event._defaultPrevented;
  }
}

if (typeof globalThis.Event !== 'function') {
  globalThis.EventTarget = EventTarget;
  globalThis.Event = Event;
  globalThis.CustomEvent = CustomEvent;
}
