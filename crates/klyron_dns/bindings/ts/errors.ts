export class DnsResolverError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'DnsResolverError';
  }
}

export class DnsResolverHttpError extends DnsResolverError {
  constructor(public status: number, message: string) {
    super(message);
    this.name = 'DnsResolverHttpError';
  }
}
