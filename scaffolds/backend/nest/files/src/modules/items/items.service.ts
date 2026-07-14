import { Injectable } from '@nestjs/common'

@Injectable()
export class ItemsService {
  private readonly items: string[] = []

  findAll(): string[] {
    return this.items
  }
}
