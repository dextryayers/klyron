import { Controller, Get } from '@overnightjs/core'
@Controller('')
export class HelloController { @Get('/') private get() { return 'Hello OvernightJS' } }
