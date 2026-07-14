declare module '*.node' {
  const exports: Record<string, unknown>;
  export default exports;
}

declare namespace NodeJS {
  interface Process {
    platform: 'linux' | 'darwin' | 'win32';
  }
}
