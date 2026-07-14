import { NapiModule, NapiLoaderConfig } from './types';

export function detectModulePath(name: string): string {
  const platform =
    process.platform === 'linux'
      ? 'linux-x64-gnu'
      : process.platform === 'darwin'
        ? 'darwin-x64'
        : 'win32-x64-msvc';
  const cwd = process.cwd();
  return `${cwd}/node_modules/${name}/${name}.${platform}.node`;
}

export function mergeLoaderConfigs(
  base: NapiLoaderConfig,
  override: Partial<NapiLoaderConfig>
): NapiLoaderConfig {
  return {
    ...base,
    ...override,
    modulePaths: [...base.modulePaths, ...(override.modulePaths || [])],
  };
}
