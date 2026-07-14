export interface LinterConfig {
  enabled: boolean;
  verbose: boolean;
}

export interface LinterResult {
  success: boolean;
  message: string;
}
