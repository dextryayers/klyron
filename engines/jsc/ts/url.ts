import type { NativeJSCBindings } from "./engine.ts";

export interface JSCParsedURL {
  href: string;
  protocol: string;
  hostname: string;
  port: string;
  pathname: string;
  search: string;
  hash: string;
  host: string;
  origin: string;
  username: string;
}

export class JSCURL {
  constructor(
    private handle: number,
    private bindings: NativeJSCBindings,
  ) {}

  parse(urlStr: string): number {
    return this.bindings.jscURLParse(this.handle, urlStr);
  }

  resolve(base: string, relative: string): string {
    const r = this.bindings.jscURLResolve(this.handle, base, relative);
    if (!r.success) throw new Error("urlResolve failed");
    return r.data ?? "";
  }

  format(urlObjHandle: number): string {
    const r = this.bindings.jscURLFormat(this.handle, urlObjHandle);
    return r;
  }

  domainToASCII(domain: string): string {
    const r = this.bindings.jscURLDomainToASCII(this.handle, domain);
    if (!r.success) throw new Error("domainToASCII failed");
    return r.data ?? "";
  }
}
