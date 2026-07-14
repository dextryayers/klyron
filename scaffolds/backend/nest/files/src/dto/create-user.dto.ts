import { IsString, IsEmail, MinLength, MaxLength, IsOptional } from 'class-validator'

export class CreateUserDto {
  @IsString()
  @MinLength(2)
  @MaxLength(50)
  name: string

  @IsEmail()
  email: string

  @IsString()
  @MinLength(8)
  @MaxLength(128)
  password: string

  @IsString()
  @IsOptional()
  @MaxLength(20)
  role?: string
}
