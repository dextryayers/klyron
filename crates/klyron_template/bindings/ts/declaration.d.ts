declare module "@klyron/template" {
  export interface TemplateConfig {
    version: string;
  }
  export class TemplateClient {
    constructor(config: TemplateConfig);
    execute(): Promise<void>;
  }
  export function loadConfig(): TemplateConfig;
  export function version(): string;
}
