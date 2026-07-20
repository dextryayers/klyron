import { EventEmitter } from "./events.js";
import { op_worker_create_thread, op_worker_send, op_worker_poll, op_worker_terminate } from "ext:core/ops";

export class Worker extends EventEmitter {
  constructor(filename, options = {}) {
    super();
    const data = options.workerData;
    const result = op_worker_create_thread(filename,
      data !== undefined ? JSON.stringify(data) : "");
    const parsed = JSON.parse(result);
    this._id = parsed.id;
    this._threadId = parsed.threadId;
    this._terminated = false;
    this._pollLoop();
  }

  _pollLoop() {
    if (this._terminated) return;
    try {
      const msgs = op_worker_poll(this._id);
      if (msgs) {
        for (const m of JSON.parse(msgs)) this.emit("message", m);
      }
    } catch (e) {}
    if (this._terminated) return;
    Promise.resolve().then(() => this._pollLoop());
  }

  postMessage(msg) {
    op_worker_send(this._id, JSON.stringify(msg));
  }

  terminate() {
    this._terminated = true;
    op_worker_terminate(this._id);
    this.emit("exit", 0);
  }

  get threadId() { return this._threadId; }
}

export const isMainThread = true;
export let parentPort = null;
export let workerData = null;
export const threadId = 0;

export default { Worker, isMainThread, parentPort, workerData, threadId };
