export interface TranspilerConfig {
  enabled: boolean;
  verbose: boolean;
}

export interface TranspilerResult {
  success: boolean;
  message: string;
}
