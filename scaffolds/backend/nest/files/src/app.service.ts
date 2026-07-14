import { Injectable } from '@nestjs/common'

@Injectable()
export class AppService {
  getHello() {
    return { message: 'Hello from {{ name }}', version: '{{ version }}' }
  }
}
