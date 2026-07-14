export interface KlyronRuntimeOptions {
  engine?: 'v8' | 'boa' | 'quickjs' | 'jsc' | 'auto';
  extensions?: string[];
}

export interface ExecResult {
  stdout: string;
  stderr: string;
  code: number;
}

export interface FSReadOptions {
  encoding?: BufferEncoding;
}

export interface FSWriteOptions {
  encoding?: BufferEncoding;
  mode?: number;
}

export interface HTTPHeaders {
  [key: string]: string;
}

export interface HTTPResponse {
  status: number;
  statusText: string;
  headers: HTTPHeaders;
  text(): Promise<string>;
  json(): Promise<any>;
  blob(): Promise<Blob>;
  arrayBuffer(): Promise<ArrayBuffer>;
}

export interface KlyronEnv {
  get(key: string): string | null;
  set(key: string, value: string): void;
  getAll(): Record<string, string>;
  has(key: string): boolean;
}

export interface KlyronProcess {
  exec(command: string, args?: string[]): Promise<ExecResult>;
  spawn(command: string, args?: string[]): any;
  readonly pid: number;
  readonly cwd: string;
  readonly platform: string;
}

export interface KlyronFS {
  read(path: string, options?: FSReadOptions): Promise<string>;
  readBuffer(path: string): Promise<ArrayBuffer>;
  write(path: string, content: string | Uint8Array, options?: FSWriteOptions): Promise<void>;
  exists(path: string): Promise<boolean>;
  list(dir?: string): Promise<string[]>;
  mkdir(dir: string): Promise<void>;
  remove(path: string): Promise<void>;
  copy(src: string, dest: string): Promise<void>;
}

export interface KlyronHTTP {
  get(url: string, headers?: HTTPHeaders): Promise<HTTPResponse>;
  post(url: string, body: any, headers?: HTTPHeaders): Promise<HTTPResponse>;
  put(url: string, body: any, headers?: HTTPHeaders): Promise<HTTPResponse>;
  del(url: string, headers?: HTTPHeaders): Promise<HTTPResponse>;
  request(method: string, url: string, options?: RequestInit): Promise<HTTPResponse>;
}

export interface IRegistryClient {
  search(query: string, limit?: number): Promise<PackageSearchResult>;
  info(name: string): Promise<PackageInfo>;
  download(name: string, version: string): Promise<PackageDownload>;
}

export interface PackageInfo {
  name: string;
  version: string;
  description?: string;
  license?: string;
  homepage?: string;
  repository?: string;
  author?: string;
  keywords: string[];
  registry: string;
}

export interface PackageSearchResult {
  results: PackageInfo[];
  total: number;
  took_ms: number;
}

export interface PackageDownload {
  name: string;
  version: string;
  data: Uint8Array;
  integrity: string;
  contentType: string;
}

export class KlyronRuntime {
  constructor(options?: KlyronRuntimeOptions);
  readonly fs: KlyronFS;
  readonly http: KlyronHTTP;
  readonly process: KlyronProcess;
  readonly env: KlyronEnv;
  readonly registry: IRegistryClient;
  version(): Promise<string>;
  eval(code: string, lang?: string): Promise<string>;
  run(path: string, lang?: string): Promise<string>;
}

export default KlyronRuntime;
