import { Controller, Post, Body, HttpCode, HttpStatus } from '@nestjs/common'
import { AuthService } from './auth.service'

@Controller('auth')
export class AuthController {
  constructor(private readonly authService: AuthService) {}

  @Post('login')
  @HttpCode(HttpStatus.OK)
  login(@Body('userId') userId: string) {
    const token = this.authService.generateToken(userId)
    return { token }
  }

  @Post('validate')
  @HttpCode(HttpStatus.OK)
  validate(@Body('token') token: string) {
    const userId = this.authService.validateToken(token)
    return { valid: true, userId }
  }

  @Post('revoke')
  @HttpCode(HttpStatus.OK)
  revoke(@Body('token') token: string) {
    this.authService.revokeToken(token)
    return { revoked: true }
  }
}
