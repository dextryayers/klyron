/// <reference no-default-lib="true" />

interface Object {
  constructor: Function;
  hasOwnProperty(v: PropertyKey): boolean;
  isPrototypeOf(v: Object): boolean;
  propertyIsEnumerable(v: PropertyKey): boolean;
  toLocaleString(): string;
  toString(): string;
  valueOf(): Object;
}

interface Function {
  prototype: any;
  readonly length: number;
  readonly name: string;
  call(this: Function, thisArg: any, ...argArray: any[]): any;
  apply(this: Function, thisArg: any, argArray?: any): any;
  bind(this: Function, thisArg: any, ...argArray: any[]): any;
  toString(): string;
}

interface String {
  readonly length: number;
  charAt(pos: number): string;
  charCodeAt(index: number): number;
  concat(...strings: string[]): string;
  includes(searchString: string, position?: number): boolean;
  endsWith(searchString: string, endPosition?: number): boolean;
  indexOf(searchString: string, position?: number): number;
  lastIndexOf(searchString: string, position?: number): number;
  localeCompare(that: string): number;
  match(regexp: string | RegExp): RegExpMatchArray | null;
  matchAll(regexp: RegExp): IterableIterator<RegExpMatchArray>;
  normalize(form?: "NFC" | "NFD" | "NFKC" | "NFKD"): string;
  padEnd(targetLength: number, fillString?: string): string;
  padStart(targetLength: number, fillString?: string): string;
  repeat(count: number): string;
  replace(searchValue: string | RegExp, replaceValue: string | ((substring: string, ...args: any[]) => string)): string;
  replaceAll(searchValue: string | RegExp, replaceValue: string | ((substring: string, ...args: any[]) => string)): string;
  search(regexp: string | RegExp): number;
  slice(start?: number, end?: number): string;
  split(separator: string | RegExp, limit?: number): string[];
  startsWith(searchString: string, position?: number): boolean;
  substring(start: number, end?: number): string;
  toLocaleLowerCase(): string;
  toLocaleUpperCase(): string;
  toLowerCase(): string;
  toUpperCase(): string;
  trim(): string;
  trimEnd(): string;
  trimLeft(): string;
  trimRight(): string;
  trimStart(): string;
  valueOf(): string;
  [index: number]: string;
}

interface StringConstructor {
  new(value?: any): String;
  (value?: any): string;
  readonly prototype: String;
  fromCharCode(...codes: number[]): string;
  fromCodePoint(...codePoints: number[]): string;
  raw(template: TemplateStringsArray, ...substitutions: any[]): string;
}

interface Number {
  toString(radix?: number): string;
  toFixed(fractionDigits?: number): string;
  toExponential(fractionDigits?: number): string;
  toPrecision(precision?: number): string;
  valueOf(): number;
  toLocaleString(locales?: string | string[], options?: Intl.NumberFormatOptions): string;
}

interface Boolean {
  valueOf(): boolean;
}

interface Array<T> {
  readonly length: number;
  [n: number]: T;
  concat(...items: ConcatArray<T>[]): T[];
  concat(...items: (T | ConcatArray<T>)[]): T[];
  copyWithin(target: number, start: number, end?: number): this;
  entries(): IterableIterator<[number, T]>;
  every<S extends T>(predicate: (value: T, index: number, array: T[]) => value is S, thisArg?: any): this is S[];
  every(predicate: (value: T, index: number, array: T[]) => unknown, thisArg?: any): boolean;
  fill(value: T, start?: number, end?: number): this;
  filter<S extends T>(predicate: (value: T, index: number, array: T[]) => value is S, thisArg?: any): S[];
  filter(predicate: (value: T, index: number, array: T[]) => unknown, thisArg?: any): T[];
  find<S extends T>(predicate: (this: void, value: T, index: number, obj: T[]) => value is S, thisArg?: any): S | undefined;
  find(predicate: (value: T, index: number, obj: T[]) => unknown, thisArg?: any): T | undefined;
  findIndex(predicate: (value: T, index: number, obj: T[]) => unknown, thisArg?: any): number;
  findLast<S extends T>(predicate: (value: T, index: number, array: T[]) => value is S, thisArg?: any): S | undefined;
  findLast(predicate: (value: T, index: number, array: T[]) => unknown, thisArg?: any): T | undefined;
  findLastIndex(predicate: (value: T, index: number, array: T[]) => unknown, thisArg?: any): number;
  flat<A, D extends number = 1>(this: A, depth?: D): FlatArray<A, D>[];
  flatMap<U, This = undefined>(callback: (this: This, value: T, index: number, array: T[]) => U | ReadonlyArray<U>, thisArg?: This): U[];
  forEach(callbackfn: (value: T, index: number, array: T[]) => void, thisArg?: any): void;
  includes(searchElement: T, fromIndex?: number): boolean;
  indexOf(searchElement: T, fromIndex?: number): number;
  join(separator?: string): string;
  keys(): IterableIterator<number>;
  lastIndexOf(searchElement: T, fromIndex?: number): number;
  map<U>(callbackfn: (value: T, index: number, array: T[]) => U, thisArg?: any): U[];
  pop(): T | undefined;
  push(...items: T[]): number;
  reduce(callbackfn: (previousValue: T, currentValue: T, currentIndex: number, array: T[]) => T): T;
  reduce(callbackfn: (previousValue: T, currentValue: T, currentIndex: number, array: T[]) => T, initialValue: T): T;
  reduce<U>(callbackfn: (previousValue: U, currentValue: T, currentIndex: number, array: T[]) => U, initialValue: U): U;
  reduceRight(callbackfn: (previousValue: T, currentValue: T, currentIndex: number, array: T[]) => T): T;
  reduceRight(callbackfn: (previousValue: T, currentValue: T, currentIndex: number, array: T[]) => T, initialValue: T): T;
  reduceRight<U>(callbackfn: (previousValue: U, currentValue: T, currentIndex: number, array: T[]) => U, initialValue: U): U;
  reverse(): T[];
  shift(): T | undefined;
  slice(start?: number, end?: number): T[];
  some(predicate: (value: T, index: number, array: T[]) => unknown, thisArg?: any): boolean;
  sort(compareFn?: (a: T, b: T) => number): this;
  splice(start: number, deleteCount?: number): T[];
  splice(start: number, deleteCount: number, ...items: T[]): T[];
  toLocaleString(): string;
  toReversed(): T[];
  toSorted(compareFn?: (a: T, b: T) => number): T[];
  toSpliced(start: number, deleteCount?: number, ...items: T[]): T[];
  toString(): string;
  unshift(...items: T[]): number;
  values(): IterableIterator<T>;
  [Symbol.iterator](): IterableIterator<T>;
}

interface Promise<T> {
  then<TResult1 = T, TResult2 = never>(
    onfulfilled?: ((value: T) => TResult1 | PromiseLike<TResult1>) | undefined | null,
    onrejected?: ((reason: any) => TResult2 | PromiseLike<TResult2>) | undefined | null,
  ): Promise<TResult1 | TResult2>;
  catch<TResult = never>(onrejected?: ((reason: any) => TResult | PromiseLike<TResult>) | undefined | null): Promise<T | TResult>;
  finally(onfinally?: (() => void) | undefined | null): Promise<T>;
}

interface PromiseConstructor {
  readonly prototype: Promise<any>;
  new<T>(executor: (resolve: (value: T | PromiseLike<T>) => void, reject: (reason?: any) => void) => void): Promise<T>;
  all<T extends readonly unknown[] | []>(values: T): Promise<{ -readonly [P in keyof T]: Awaited<T[P]> }>;
  allSettled<T extends readonly unknown[] | []>(values: T): Promise<{ -readonly [P in keyof T]: PromiseSettledResult<Awaited<T[P]>> }>;
  race<T extends readonly unknown[] | []>(values: T): Promise<Awaited<T[number]>>;
  reject<T = never>(reason?: any): Promise<T>;
  resolve<T>(value: T | PromiseLike<T>): Promise<Awaited<T>>;
  withResolvers<T>(): { promise: Promise<T>; resolve: (value: T | PromiseLike<T>) => void; reject: (reason?: any) => void };
  any<T extends readonly unknown[] | []>(values: T): Promise<Awaited<T[number]>>;
}

declare var Promise: PromiseConstructor;

interface PromiseLike<T> {
  then<TResult1 = T, TResult2 = never>(
    onfulfilled?: ((value: T) => TResult1 | PromiseLike<TResult1>) | undefined | null,
    onrejected?: ((reason: any) => TResult2 | PromiseLike<TResult2>) | undefined | null,
  ): PromiseLike<TResult1 | TResult2>;
}

interface RegExp {
  readonly lastIndex: number;
  exec(string: string): RegExpExecArray | null;
  test(string: string): boolean;
  readonly dotAll: boolean;
  readonly flags: string;
  readonly global: boolean;
  readonly hasIndices: boolean;
  readonly ignoreCase: boolean;
  readonly multiline: boolean;
  readonly source: string;
  readonly sticky: boolean;
  readonly unicode: boolean;
  readonly unicodeSets: boolean;
  toString(): string;
  [Symbol.match](string: string): RegExpMatchArray | null;
  [Symbol.matchAll](string: string): IterableIterator<RegExpMatchArray>;
  [Symbol.replace](string: string, replaceValue: string): string;
  [Symbol.search](string: string): number;
  [Symbol.split](string: string, limit?: number): string[];
}

interface Error {
  name: string;
  message: string;
  stack?: string;
  cause?: unknown;
}

interface TypeError extends Error {}
interface RangeError extends Error {}
interface ReferenceError extends Error {}
interface SyntaxError extends Error {}
interface EvalError extends Error {}
interface URIError extends Error {}

interface Date {
  getDate(): number;
  getDay(): number;
  getFullYear(): number;
  getHours(): number;
  getMilliseconds(): number;
  getMinutes(): number;
  getMonth(): number;
  getSeconds(): number;
  getTime(): number;
  getTimezoneOffset(): number;
  getUTCDate(): number;
  getUTCDay(): number;
  getUTCFullYear(): number;
  getUTCHours(): number;
  getUTCMilliseconds(): number;
  getUTCMinutes(): number;
  getUTCMonth(): number;
  getUTCSeconds(): number;
  setDate(date: number): number;
  setFullYear(year: number, month?: number, date?: number): number;
  setHours(hours: number, min?: number, sec?: number, ms?: number): number;
  setMilliseconds(ms: number): number;
  setMinutes(min: number, sec?: number, ms?: number): number;
  setMonth(month: number, date?: number): number;
  setSeconds(sec: number, ms?: number): number;
  setTime(time: number): number;
  setUTCDate(date: number): number;
  setUTCFullYear(year: number, month?: number, date?: number): number;
  setUTCHours(hours: number, min?: number, sec?: number, ms?: number): number;
  setUTCMilliseconds(ms: number): number;
  setUTCMinutes(min: number, sec?: number, ms?: number): number;
  setUTCMonth(month: number, date?: number): number;
  setUTCSeconds(sec: number, ms?: number): number;
  toDateString(): string;
  toISOString(): string;
  toJSON(key?: any): string;
  toLocaleDateString(locales?: string | string[], options?: Intl.DateTimeFormatOptions): string;
  toLocaleString(locales?: string | string[], options?: Intl.DateTimeFormatOptions): string;
  toLocaleTimeString(locales?: string | string[], options?: Intl.DateTimeFormatOptions): string;
  toString(): string;
  toTimeString(): string;
  toUTCString(): string;
  valueOf(): number;
  [Symbol.toPrimitive](hint: "default" | "string" | "number"): string | number;
}

interface Math {
  E: number;
  LN10: number;
  LN2: number;
  LOG10E: number;
  LOG2E: number;
  PI: number;
  SQRT1_2: number;
  SQRT2: number;
  abs(x: number): number;
  acos(x: number): number;
  acosh(x: number): number;
  asin(x: number): number;
  asinh(x: number): number;
  atan(x: number): number;
  atan2(y: number, x: number): number;
  atanh(x: number): number;
  cbrt(x: number): number;
  ceil(x: number): number;
  clz32(x: number): number;
  cos(x: number): number;
  cosh(x: number): number;
  exp(x: number): number;
  expm1(x: number): number;
  floor(x: number): number;
  fround(x: number): number;
  hypot(...values: number[]): number;
  imul(x: number, y: number): number;
  log(x: number): number;
  log10(x: number): number;
  log1p(x: number): number;
  log2(x: number): number;
  max(...values: number[]): number;
  min(...values: number[]): number;
  pow(x: number, y: number): number;
  random(): number;
  round(x: number): number;
  sign(x: number): number;
  sin(x: number): number;
  sinh(x: number): number;
  sqrt(x: number): number;
  tan(x: number): number;
  tanh(x: number): number;
  trunc(x: number): number;
}

declare var Math: Math;

interface JSON {
  parse(text: string, reviver?: (this: any, key: string, value: any) => any): any;
  stringify(value: any, replacer?: (this: any, key: string, value: any) => any, space?: string | number): string;
  stringify(value: any, replacer?: (number | string)[] | null, space?: string | number): string;
}

declare var JSON: JSON;

// Web API types

interface EventTarget {
  addEventListener(type: string, callback: EventListenerOrEventListenerObject | null, options?: AddEventListenerOptions | boolean): void;
  dispatchEvent(event: Event): boolean;
  removeEventListener(type: string, callback: EventListenerOrEventListenerObject | null, options?: EventListenerOptions | boolean): void;
}

interface EventListener {
  (evt: Event): void;
}

interface EventListenerObject {
  handleEvent(object: Event): void;
}

interface Event {
  readonly type: string;
  readonly target: EventTarget | null;
  readonly currentTarget: EventTarget | null;
  readonly bubbles: boolean;
  readonly cancelable: boolean;
  readonly defaultPrevented: boolean;
  readonly eventPhase: number;
  readonly isTrusted: boolean;
  readonly timeStamp: DOMHighResTimeStamp;
  composedPath(): EventTarget[];
  initEvent(type: string, bubbles?: boolean, cancelable?: boolean): void;
  preventDefault(): void;
  stopImmediatePropagation(): void;
  stopPropagation(): void;
  readonly AT_TARGET: number;
  readonly BUBBLING_PHASE: number;
  readonly CAPTURING_PHASE: number;
  readonly NONE: number;
}

interface EventInit {
  bubbles?: boolean;
  cancelable?: boolean;
  composed?: boolean;
}

interface CustomEvent<T = any> extends Event {
  readonly detail: T;
}

interface CustomEventInit<T = any> extends EventInit {
  detail?: T;
}

declare var CustomEvent: {
  new<T>(type: string, eventInitDict?: CustomEventInit<T>): CustomEvent<T>;
  prototype: CustomEvent;
};

interface AbortController {
  readonly signal: AbortSignal;
  abort(reason?: any): void;
}

declare var AbortController: {
  new(): AbortController;
  prototype: AbortController;
};

interface AbortSignal extends EventTarget {
  readonly aborted: boolean;
  readonly reason: any;
  throwIfAborted(): void;
  onabort: ((this: AbortSignal, ev: Event) => any) | null;
}

interface AbortSignal {
  timeout(milliseconds: number): AbortSignal;
}

declare var AbortSignal: {
  prototype: AbortSignal;
  new(): AbortSignal;
  abort(reason?: any): AbortSignal;
  timeout(milliseconds: number): AbortSignal;
};

interface Blob {
  readonly size: number;
  readonly type: string;
  arrayBuffer(): Promise<ArrayBuffer>;
  slice(start?: number, end?: number, contentType?: string): Blob;
  stream(): ReadableStream;
  text(): Promise<string>;
  bytes(): Promise<Uint8Array>;
}

declare var Blob: {
  new(blobParts?: BlobPart[], options?: BlobPropertyBag): Blob;
  prototype: Blob;
};

interface File extends Blob {
  readonly lastModified: number;
  readonly name: string;
  readonly webkitRelativePath: string;
}

declare var File: {
  new(fileBits: BlobPart[], fileName: string, options?: FilePropertyBag): File;
  prototype: File;
};

interface TextDecoder {
  readonly encoding: string;
  readonly fatal: boolean;
  readonly ignoreBOM: boolean;
  decode(input?: BufferSource, options?: TextDecodeOptions): string;
}

declare var TextDecoder: {
  new(label?: string, options?: TextDecoderOptions): TextDecoder;
  prototype: TextDecoder;
};

interface TextEncoder {
  readonly encoding: string;
  encode(input?: string): Uint8Array;
  encodeInto(input: string, dest: Uint8Array): TextEncoderEncodeIntoResult;
}

declare var TextEncoder: {
  new(): TextEncoder;
  prototype: TextEncoder;
};

interface URL {
  hash: string;
  host: string;
  hostname: string;
  href: string;
  readonly origin: string;
  password: string;
  pathname: string;
  port: string;
  protocol: string;
  search: string;
  readonly searchParams: URLSearchParams;
  username: string;
  toJSON(): string;
  toString(): string;
}

declare var URL: {
  new(url: string | URL, base?: string | URL): URL;
  prototype: URL;
  createObjectURL(obj: Blob | MediaSource): string;
  revokeObjectURL(url: string): void;
  canParse(url: string, base?: string): boolean;
};

interface URLSearchParams {
  readonly size: number;
  append(name: string, value: string): void;
  delete(name: string): void;
  entries(): IterableIterator<[string, string]>;
  forEach(callbackfn: (value: string, key: string, parent: URLSearchParams) => void, thisArg?: any): void;
  get(name: string): string | null;
  getAll(name: string): string[];
  has(name: string): boolean;
  keys(): IterableIterator<string>;
  set(name: string, value: string): void;
  sort(): void;
  toString(): string;
  values(): IterableIterator<string>;
  [Symbol.iterator](): IterableIterator<[string, string]>;
}

declare var URLSearchParams: {
  new(init?: string[][] | Record<string, string> | string | URLSearchParams): URLSearchParams;
  prototype: URLSearchParams;
};

interface Headers {
  append(name: string, value: string): void;
  delete(name: string): void;
  entries(): IterableIterator<[string, string]>;
  forEach(callbackfn: (value: string, key: string, parent: Headers) => void, thisArg?: any): void;
  get(name: string): string | null;
  has(name: string): boolean;
  keys(): IterableIterator<string>;
  set(name: string, value: string): void;
  values(): IterableIterator<string>;
  [Symbol.iterator](): IterableIterator<[string, string]>;
  getSetCookie(): string[];
}

declare var Headers: {
  new(init?: HeadersInit | Headers): Headers;
  prototype: Headers;
};

interface Body {
  readonly body: ReadableStream<Uint8Array> | null;
  readonly bodyUsed: boolean;
  arrayBuffer(): Promise<ArrayBuffer>;
  blob(): Promise<Blob>;
  formData(): Promise<FormData>;
  json(): Promise<any>;
  text(): Promise<string>;
  bytes(): Promise<Uint8Array>;
}

interface Request extends Body {
  readonly cache: RequestCache;
  readonly credentials: RequestCredentials;
  readonly destination: RequestDestination;
  readonly headers: Headers;
  readonly integrity: string;
  readonly keepalive: boolean;
  readonly method: string;
  readonly mode: RequestMode;
  readonly redirect: RequestRedirect;
  readonly referrer: string;
  readonly referrerPolicy: ReferrerPolicy;
  readonly signal: AbortSignal;
  readonly url: string;
  clone(): Request;
}

declare var Request: {
  new(input: RequestInfo | URL, init?: RequestInit): Request;
  prototype: Request;
};

interface Response extends Body {
  readonly headers: Headers;
  readonly ok: boolean;
  readonly redirected: boolean;
  readonly status: number;
  readonly statusText: string;
  readonly type: ResponseType;
  readonly url: string;
  clone(): Response;
  static error(): Response;
  static json(data: any, init?: ResponseInit): Response;
  static redirect(url: string | URL, status?: number): Response;
}

declare var Response: {
  new(body?: BodyInit | null, init?: ResponseInit): Response;
  prototype: Response;
  error(): Response;
  json(data: any, init?: ResponseInit): Response;
  redirect(url: string | URL, status?: number): Response;
};

interface ReadableStream<R = any> {
  readonly locked: boolean;
  cancel(reason?: any): Promise<void>;
  getReader(options: { mode: "byob" }): ReadableStreamBYOBReader;
  getReader(): ReadableStreamDefaultReader<R>;
  pipeThrough<T>(transform: ReadableWritablePair<T, R>, options?: StreamPipeOptions): ReadableStream<T>;
  pipeTo(dest: WritableStream<R>, options?: StreamPipeOptions): Promise<void>;
  tee(): [ReadableStream<R>, ReadableStream<R>];
  values(options?: { preventCancel?: boolean }): AsyncIterableIterator<R>;
  [Symbol.asyncIterator](options?: { preventCancel?: boolean }): AsyncIterableIterator<R>;
}

declare var ReadableStream: {
  new<R = any>(underlyingSource?: UnderlyingSource<R>, strategy?: QueuingStrategy<R>): ReadableStream<R>;
  prototype: ReadableStream;
};

interface WritableStream<R = any> {
  readonly locked: boolean;
  abort(reason?: any): Promise<void>;
  close(): Promise<void>;
  getWriter(): WritableStreamDefaultWriter<R>;
}

declare var WritableStream: {
  new<R = any>(underlyingSink?: UnderlyingSink<R>, strategy?: QueuingStrategy<R>): WritableStream<R>;
  prototype: WritableStream;
};

interface TransformStream<I = any, O = any> {
  readonly readable: ReadableStream<O>;
  readonly writable: WritableStream<I>;
}

declare var TransformStream: {
  new<I = any, O = any>(transformer?: Transformer<I, O>, writableStrategy?: QueuingStrategy<I>, readableStrategy?: QueuingStrategy<O>): TransformStream<I, O>;
  prototype: TransformStream;
};

interface MessageEvent<T = any> extends Event {
  readonly data: T;
  readonly origin: string;
  readonly lastEventId: string;
  readonly source: MessageEventSource | null;
  readonly ports: ReadonlyArray<MessagePort>;
}

interface MessagePort extends EventTarget {
  close(): void;
  postMessage(message: any, transfer?: Transferable[]): void;
  start(): void;
  onmessage: ((this: MessagePort, ev: MessageEvent) => any) | null;
  onmessageerror: ((this: MessagePort, ev: MessageEvent) => any) | null;
}

interface WebSocket extends EventTarget {
  readonly url: string;
  readonly readyState: number;
  readonly protocol: string;
  readonly extensions: string;
  readonly bufferedAmount: number;
  readonly binaryType: BinaryType;
  close(code?: number, reason?: string): void;
  send(data: string | ArrayBufferLike | Blob | ArrayBufferView): void;
  onopen: ((this: WebSocket, ev: Event) => any) | null;
  onclose: ((this: WebSocket, ev: CloseEvent) => any) | null;
  onerror: ((this: WebSocket, ev: Event) => any) | null;
  onmessage: ((this: WebSocket, ev: MessageEvent) => any) | null;
  readonly CONNECTING: number;
  readonly OPEN: number;
  readonly CLOSING: number;
  readonly CLOSED: number;
}

declare var WebSocket: {
  new(url: string | URL, protocols?: string | string[]): WebSocket;
  prototype: WebSocket;
  readonly CONNECTING: number;
  readonly OPEN: number;
  readonly CLOSING: number;
  readonly CLOSED: number;
};

interface CloseEvent extends Event {
  readonly code: number;
  readonly reason: string;
  readonly wasClean: boolean;
}

interface MessageChannel {
  readonly port1: MessagePort;
  readonly port2: MessagePort;
}

declare var MessageChannel: {
  new(): MessageChannel;
  prototype: MessageChannel;
};

interface FormData {
  append(name: string, value: string | Blob, fileName?: string): void;
  delete(name: string): void;
  entries(): IterableIterator<[string, string | File]>;
  forEach(callbackfn: (value: string | File, key: string, parent: FormData) => void, thisArg?: any): void;
  get(name: string): string | File | null;
  getAll(name: string): string | File[];
  has(name: string): boolean;
  keys(): IterableIterator<string>;
  set(name: string, value: string | Blob, fileName?: string): void;
  values(): IterableIterator<string | File>;
  [Symbol.iterator](): IterableIterator<[string, string | File]>;
}

declare var FormData: {
  new(form?: HTMLFormElement): FormData;
  prototype: FormData;
};

interface Performance {
  readonly timeOrigin: number;
  clearMarks(markName?: string): void;
  clearMeasures(measureName?: string): void;
  clearResourceTimings(): void;
  getEntries(): PerformanceEntryList;
  getEntriesByName(name: string, type?: string): PerformanceEntryList;
  getEntriesByType(type: string): PerformanceEntryList;
  mark(markName: string, markOptions?: PerformanceMarkOptions): PerformanceMark;
  measure(measureName: string, startOrMeasureOptions?: string | PerformanceMeasureOptions, endMark?: string): PerformanceMeasure;
  now(): number;
  toJSON(): any;
}

declare var performance: Performance;

interface PerformanceEntry {
  readonly name: string;
  readonly entryType: string;
  readonly startTime: DOMHighResTimeStamp;
  readonly duration: DOMHighResTimeStamp;
  toJSON(): any;
}

interface PerformanceMark extends PerformanceEntry {
  readonly detail: any;
}

interface PerformanceMeasure extends PerformanceEntry {
  readonly detail: any;
}

interface Storage {
  readonly length: number;
  clear(): void;
  getItem(key: string): string | null;
  key(index: number): string | null;
  removeItem(key: string): void;
  setItem(key: string, value: string): void;
  [name: string]: any;
}

declare var localStorage: Storage;
declare var sessionStorage: Storage;

interface Navigator {
  readonly hardwareConcurrency: number;
  readonly language: string;
  readonly languages: ReadonlyArray<string>;
  readonly platform: string;
  readonly userAgent: string;
}

declare var navigator: Navigator;

// Crypto API

interface Crypto {
  readonly subtle: SubtleCrypto;
  getRandomValues<T extends ArrayBufferView | null>(array: T): T;
  randomUUID(): string;
}

declare var crypto: Crypto;

interface SubtleCrypto {
  digest(algorithm: AlgorithmIdentifier, data: BufferSource): Promise<ArrayBuffer>;
  encrypt(algorithm: AlgorithmIdentifier, key: CryptoKey, data: BufferSource): Promise<ArrayBuffer>;
  decrypt(algorithm: AlgorithmIdentifier, key: CryptoKey, data: BufferSource): Promise<ArrayBuffer>;
  sign(algorithm: AlgorithmIdentifier, key: CryptoKey, data: BufferSource): Promise<ArrayBuffer>;
  verify(algorithm: AlgorithmIdentifier, key: CryptoKey, signature: BufferSource, data: BufferSource): Promise<boolean>;
  generateKey(algorithm: AlgorithmIdentifier, extractable: boolean, keyUsages: KeyUsage[]): Promise<CryptoKeyPair | CryptoKey>;
  importKey(format: KeyFormat, keyData: BufferSource | JsonWebKey, algorithm: AlgorithmIdentifier, extractable: boolean, keyUsages: KeyUsage[]): Promise<CryptoKey>;
  exportKey(format: "raw" | "pkcs8" | "spki" | "jwk", key: CryptoKey): Promise<ArrayBuffer | JsonWebKey>;
  wrapKey(format: KeyFormat, key: CryptoKey, wrappingKey: CryptoKey, wrapAlgorithm: AlgorithmIdentifier): Promise<ArrayBuffer>;
  unwrapKey(format: KeyFormat, wrappedKey: BufferSource, unwrappingKey: CryptoKey, unwrapAlgorithm: AlgorithmIdentifier, unwrappedKeyAlgorithm: AlgorithmIdentifier, extractable: boolean, keyUsages: KeyUsage[]): Promise<CryptoKey>;
  deriveBits(algorithm: AlgorithmIdentifier, baseKey: CryptoKey, length: number): Promise<ArrayBuffer>;
  deriveKey(algorithm: AlgorithmIdentifier, baseKey: CryptoKey, derivedKeyType: AlgorithmIdentifier, extractable: boolean, keyUsages: KeyUsage[]): Promise<CryptoKey>;
}

interface CryptoKey {
  readonly type: KeyType;
  readonly extractable: boolean;
  readonly algorithm: KeyAlgorithm;
  readonly usages: KeyUsage[];
}

// Timing-safe helper types

type DOMHighResTimeStamp = number;
type BufferSource = ArrayBufferView | ArrayBuffer;
type BlobPart = Blob | string | BufferSource;
type HeadersInit = Headers | string[][] | Record<string, string>;
type BodyInit = Blob | BufferSource | FormData | URLSearchParams | ReadableStream | string;
type RequestInfo = Request | string;
type BinaryType = "blob" | "arraybuffer";
type ReferrerPolicy = "" | "no-referrer" | "no-referrer-when-downgrade" | "origin" | "origin-when-cross-origin" | "same-origin" | "strict-origin" | "strict-origin-when-cross-origin" | "unsafe-url";
type RequestCache = "default" | "force-cache" | "no-cache" | "no-store" | "only-if-cached" | "reload";
type RequestCredentials = "include" | "omit" | "same-origin";
type RequestDestination = "" | "audio" | "audioworklet" | "document" | "embed" | "font" | "frame" | "frameborder" | "iframe" | "image" | "json" | "manifest" | "object" | "paintworklet" | "report" | "script" | "sharedworker" | "style" | "track" | "video" | "worker" | "xslt";
type RequestMode = "cors" | "navigate" | "no-cors" | "same-origin";
type RequestRedirect = "error" | "follow" | "manual";
type ResponseType = "basic" | "cors" | "default" | "error" | "opaque" | "opaqueredirect";
type KeyFormat = "jwk" | "pkcs8" | "raw" | "spki";
type KeyType = "private" | "public" | "secret";
type KeyUsage = "decrypt" | "deriveBits" | "deriveKey" | "encrypt" | "sign" | "unwrapKey" | "verify" | "wrapKey";
type AlgorithmIdentifier = Algorithm | string;

interface Algorithm {
  name: string;
}

interface ResponseInit {
  headers?: HeadersInit;
  status?: number;
  statusText?: string;
}

interface RequestInit {
  body?: BodyInit | null;
  cache?: RequestCache;
  credentials?: RequestCredentials;
  headers?: HeadersInit;
  integrity?: string;
  keepalive?: boolean;
  method?: string;
  mode?: RequestMode;
  redirect?: RequestRedirect;
  referrer?: string;
  referrerPolicy?: ReferrerPolicy;
  signal?: AbortSignal | null;
  window?: any;
}

interface TextDecodeOptions {
  stream?: boolean;
}

interface TextDecoderOptions {
  fatal?: boolean;
  ignoreBOM?: boolean;
}

interface TextEncoderEncodeIntoResult {
  read: number;
  written: number;
}
