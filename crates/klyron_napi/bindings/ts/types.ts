export interface NapiModule {
  name: string;
  exports: Record<string, unknown>;
}

export interface NapiLoaderConfig {
  modulePaths: string[];
  cacheEnabled: boolean;
}

export interface NapiVersion {
  major: number;
  minor: number;
  patch: number;
}

export interface NapiModuleInfo {
  name: string;
  loaded: boolean;
  symbolCount: number;
}
