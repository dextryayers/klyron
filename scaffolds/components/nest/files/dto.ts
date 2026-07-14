import { IsString, IsOptional, IsNumber, Min, Max } from 'class-validator'

export class CreateItemDto {
  @IsString()
  name: string

  @IsOptional()
  @IsString()
  description?: string

  @IsOptional()
  @IsNumber()
  @Min(0)
  price?: number
}

export class UpdateItemDto {
  @IsOptional()
  @IsString()
  name?: string

  @IsOptional()
  @IsString()
  description?: string

  @IsOptional()
  @IsNumber()
  @Min(0)
  price?: number
}
