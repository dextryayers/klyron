// Klyron Runtime — node:v8 polyfill

function getHeapStatistics() {
  return {
    total_heap_size: 0,
    total_heap_size_executable: 0,
    total_physical_size: 0,
    total_available_size: 0,
    used_heap_size: 0,
    heap_size_limit: 0,
    malloced_memory: 0,
    peak_malloced_memory: 0,
    does_zap_garbage: 0,
  };
}

function getHeapSpaceStatistics() {
  const spaces = ['new_space', 'old_space', 'code_space', 'map_space', 'large_object_space', 'new_large_object_space'];
  return spaces.map(space_name => ({
    space_name,
    space_size: 0,
    space_used_size: 0,
    space_available_size: 0,
    physical_space_size: 0,
  }));
}

function setFlagsFromString(flags) {
  try {
    if (typeof Klyron !== 'undefined' && typeof Klyron.v8SetFlags === 'function') {
      Klyron.v8SetFlags(flags);
    }
  } catch (e) {}
}

function getHeapCodeStatistics() {
  return {
    code_and_metadata_size: 0,
    bytecode_and_metadata_size: 0,
    external_script_source_size: 0,
  };
}

function serialize(value) {
  if (typeof globalThis.structuredClone !== 'function') {
    const json = JSON.stringify(value);
    return new TextEncoder().encode(json);
  }
  try {
    const clone = globalThis.structuredClone(value);
    const json = JSON.stringify(clone);
    return new TextEncoder().encode(json);
  } catch (e) {
    const json = JSON.stringify(value);
    return new TextEncoder().encode(json);
  }
}

function deserialize(buffer) {
  if (typeof buffer === 'string') buffer = new TextEncoder().encode(buffer);
  const decoder = new TextDecoder();
  const str = decoder.decode(buffer);
  return JSON.parse(str);
}

function getHeapSnapshot() {
  const EventEmitter = (typeof require !== 'undefined' ? require('events') : globalThis).EventEmitter;
  const snapshot = new (EventEmitter || Object)();
  snapshot.read = () => null;
  snapshot._read = () => {};
  return snapshot;
}

const v8 = {
  getHeapStatistics,
  getHeapSpaceStatistics,
  setFlagsFromString,
  getHeapCodeStatistics,
  serialize,
  deserialize,
  getHeapSnapshot,
  defaultOptions: {},
  setHeapSnapshotNearHeapLimit: () => {},
  stopCoverage: () => {},
  takeCoverage: () => {},
};

if (typeof module !== 'undefined' && module.exports) {
  module.exports = v8;
}
