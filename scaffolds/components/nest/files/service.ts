import { Injectable, NotFoundException } from '@nestjs/common'
import { InjectRepository } from '@nestjs/typeorm'
import { Repository } from 'typeorm'
import { Item } from './entities/item.entity'
import { CreateItemDto } from './dto/create-item.dto'
import { UpdateItemDto } from './dto/update-item.dto'

@Injectable()
export class ItemsService {
  constructor(
    @InjectRepository(Item)
    private readonly itemRepository: Repository<Item>,
  ) {}

  async findAll(): Promise<Item[]> {
    return this.itemRepository.find()
  }

  async findOne(id: number): Promise<Item> {
    const item = await this.itemRepository.findOne({ where: { id } })
    if (!item) throw new NotFoundException(`Item #${id} not found`)
    return item
  }

  async create(createItemDto: CreateItemDto): Promise<Item> {
    const item = this.itemRepository.create(createItemDto)
    return this.itemRepository.save(item)
  }

  async update(id: number, updateItemDto: UpdateItemDto): Promise<Item> {
    const item = await this.findOne(id)
    Object.assign(item, updateItemDto)
    return this.itemRepository.save(item)
  }

  async remove(id: number): Promise<void> {
    const item = await this.findOne(id)
    await this.itemRepository.remove(item)
  }
}
