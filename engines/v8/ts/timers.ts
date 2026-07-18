export interface TimerRef {
  id: number
  _ref: boolean
}

const timerCallbacks = new Map<number, (...args: unknown[]) => void>()
const timerIntervals = new Map<number, number>()
let nextTimerId = 1

function scheduleTimer(cb: (...args: unknown[]) => void, ms: number, repeat: boolean): TimerRef {
  const id = nextTimerId++
  timerCallbacks.set(id, cb)
  if (repeat) timerIntervals.set(id, ms)

  const execute = () => {
    const cb = timerCallbacks.get(id)
    if (cb) {
      cb()
      if (repeat) {
        setTimeout(() => scheduleTimer(cb, ms, true), ms)
      } else {
        timerCallbacks.delete(id)
      }
    }
  }

  if (ms === 0) {
    queueMicrotask(execute)
  } else {
    setTimeout(execute, ms)
  }

  return { id, _ref: true }
}

export function setTimeout(cb: (...args: unknown[]) => void, ms: number, ..._args: unknown[]): TimerRef {
  return scheduleTimer(cb, ms, false)
}

export function clearTimeout(ref: TimerRef | number): void {
  const id = typeof ref === "number" ? ref : ref.id
  timerCallbacks.delete(id)
  timerIntervals.delete(id)
}

export function setInterval(cb: (...args: unknown[]) => void, ms: number, ..._args: unknown[]): TimerRef {
  const ref = scheduleTimer(cb, ms, true)
  timerIntervals.set(ref.id, ms)
  return ref
}

export function clearInterval(ref: TimerRef | number): void {
  clearTimeout(ref)
}

const immediateQueue: Array<{ id: number; cb: () => void }> = []
let immediateId = 1
let immediateScheduled = false

function processImmediateQueue() {
  immediateScheduled = false
  const items = immediateQueue.splice(0)
  for (const item of items) {
    try {
      item.cb()
    } catch (e) {
      queueMicrotask(() => { throw e })
    }
  }
}

export function setImmediate(cb: (...args: unknown[]) => void, ..._args: unknown[]): TimerRef {
  const id = immediateId++
  immediateQueue.push({ id, cb: cb as () => void })
  if (!immediateScheduled) {
    immediateScheduled = true
    queueMicrotask(processImmediateQueue)
  }
  return { id, _ref: true }
}

export function clearImmediate(ref: TimerRef | number): void {
  const id = typeof ref === "number" ? ref : ref.id
  const idx = immediateQueue.findIndex(item => item.id === id)
  if (idx >= 0) immediateQueue.splice(idx, 1)
}

export function _runTimers(): void {
  processImmediateQueue()
}
