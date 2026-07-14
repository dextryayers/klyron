export interface TestConfig {
  enabled: boolean;
  verbose: boolean;
}

export interface TestResult {
  success: boolean;
  message: string;
}
