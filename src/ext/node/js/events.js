export class EventEmitter {
  constructor() { this._events = {}; }

  on(event, listener) {
    if (!this._events[event]) this._events[event] = [];
    this._events[event].push(listener);
    return this;
  }

  addListener(event, listener) { return this.on(event, listener); }

  emit(event, ...args) {
    const listeners = this._events[event];
    if (!listeners) return false;
    for (const fn of [...listeners]) fn(...args);
    return true;
  }

  once(event, listener) {
    const wrapper = (...args) => { this.off(event, wrapper); listener(...args); };
    wrapper.original = listener;
    this.on(event, wrapper);
    return this;
  }

  off(event, listener) {
    const listeners = this._events[event];
    if (!listeners) return this;
    this._events[event] = listeners.filter(l => l !== listener && l.original !== listener);
    return this;
  }

  removeListener(event, listener) { return this.off(event, listener); }

  removeAllListeners(event) {
    if (event) delete this._events[event];
    else this._events = {};
    return this;
  }

  listeners(event) { return [...(this._events[event] || [])]; }
  rawListeners(event) { return this.listeners(event); }
  listenerCount(event) { return (this._events[event] || []).length; }
  eventNames() { return Object.keys(this._events); }

  prependListener(event, listener) {
    if (!this._events[event]) this._events[event] = [];
    this._events[event].unshift(listener);
    return this;
  }

  prependOnceListener(event, listener) {
    const wrapper = (...args) => { this.off(event, wrapper); listener(...args); };
    wrapper.original = listener;
    this.prependListener(event, wrapper);
    return this;
  }
}

export default EventEmitter;
