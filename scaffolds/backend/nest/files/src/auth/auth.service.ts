import { Injectable, UnauthorizedException } from '@nestjs/common'

@Injectable()
export class AuthService {
  private readonly apiKeys = new Map<string, string>()

  validateApiKey(apiKey: string): boolean {
    return this.apiKeys.has(apiKey)
  }

  generateToken(userId: string): string {
    const token = Buffer.from(`${userId}:${Date.now()}`).toString('base64')
    this.apiKeys.set(token, userId)
    return token
  }

  validateToken(token: string): string {
    const userId = this.apiKeys.get(token)
    if (!userId) {
      throw new UnauthorizedException('Invalid token')
    }
    return userId
  }

  revokeToken(token: string): void {
    this.apiKeys.delete(token)
  }
}
