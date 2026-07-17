import { Controller, Get } from '@foal/core'
@Controller('/')
export class AppController { @Get('/') index() { return 'Hello FoalTS' } }
