// Klyron Runtime — node:diagnostics_channel polyfill

const channels = new Map();

class Channel {
  constructor(name) {
    this.name = name;
    this._subscribers = new Set();
    this._stores = new Set();
  }

  get subscribers() {
    const that = this;
    return {
      get size() { return that._subscribers.size; },
    };
  }

  subscribe(listener) {
    if (typeof listener !== 'function') throw new TypeError('listener must be a function');
    this._subscribers.add(listener);
    return () => this.unsubscribe(listener);
  }

  unsubscribe(listener) {
    this._subscribers.delete(listener);
  }

  publish(data) {
    for (const subscriber of this._subscribers) {
      try {
        subscriber(data, this.name);
      } catch (e) {
        queueMicrotask(() => { throw e; });
      }
    }
  }

  bindStore(store) {
    if (store && typeof store.runWith === 'function') {
      this._stores.add(store);
    }
  }

  unbindStore(store) {
    this._stores.delete(store);
  }
}

function channel(name) {
  if (!channels.has(name)) {
    channels.set(name, new Channel(name));
  }
  return channels.get(name);
}

function hasSubscribers(name) {
  const ch = channels.get(name);
  return ch && ch._subscribers.size > 0;
}

function subscribe(name, listener) {
  return channel(name).subscribe(listener);
}

function unsubscribe(name, listener) {
  const ch = channels.get(name);
  if (ch) ch.unsubscribe(listener);
}

class TracingChannel {
  constructor(name) {
    this.start = channel(`tracing:${name}:start`);
    this.end = channel(`tracing:${name}:end`);
    this.asyncStart = channel(`tracing:${name}:asyncStart`);
    this.asyncEnd = channel(`tracing:${name}:asyncEnd`);
    this.error = channel(`tracing:${name}:error`);
    this._name = name;
  }

  get name() { return this._name; }

  subscribe(handlers) {
    const subs = [];
    if (handlers.start) subs.push(this.start.subscribe(handlers.start));
    if (handlers.end) subs.push(this.end.subscribe(handlers.end));
    if (handlers.asyncStart) subs.push(this.asyncStart.subscribe(handlers.asyncStart));
    if (handlers.asyncEnd) subs.push(this.asyncEnd.subscribe(handlers.asyncEnd));
    if (handlers.error) subs.push(this.error.subscribe(handlers.error));
    return () => subs.forEach(unsub => unsub());
  }

  unsubscribe(handlers) {
    if (handlers.start) this.start.unsubscribe(handlers.start);
    if (handlers.end) this.end.unsubscribe(handlers.end);
    if (handlers.asyncStart) this.asyncStart.unsubscribe(handlers.asyncStart);
    if (handlers.asyncEnd) this.asyncEnd.unsubscribe(handlers.asyncEnd);
    if (handlers.error) this.error.unsubscribe(handlers.error);
  }

  traceSync(fn, context = {}, data) {
    this.start.publish(data);
    try {
      const result = fn(context);
      this.end.publish(data);
      return result;
    } catch (err) {
      this.error.publish({ ...data, error: err });
      this.end.publish(data);
      throw err;
    }
  }

  async tracePromise(fn, context = {}, data) {
    this.start.publish(data);
    try {
      const result = await fn(context);
      this.end.publish(data);
      return result;
    } catch (err) {
      this.error.publish({ ...data, error: err });
      this.end.publish(data);
      throw err;
    }
  }
}

function tracingChannel(name) {
  return new TracingChannel(name);
}

const diagnostics_channel = {
  channel,
  hasSubscribers,
  subscribe,
  unsubscribe,
  tracingChannel,
  Channel,
  TracingChannel,
};

if (typeof module !== 'undefined' && module.exports) {
  module.exports = diagnostics_channel;
}
