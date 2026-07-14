import { WatcherConfig } from './types';

export const DEFAULT_WATCHER_CONFIG: WatcherConfig = {
  enabled: true,
  verbose: false,
};

export function createWatcherConfig(overrides?: Partial<WatcherConfig>): WatcherConfig {
  return { ...DEFAULT_WATCHER_CONFIG, ...overrides };
}
