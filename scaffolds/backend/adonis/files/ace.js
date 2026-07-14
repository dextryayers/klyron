import 'reflect-metadata'
import { Ignitor } from '@adonisjs/core'
import { defineConfig } from '@adonisjs/core/assembler'

const ignitor = Ignitor.makeApp(defineConfig({
  commands: () => import('@adonisjs/core/commands'),
  ace: {
    serializers: () => import('@adonisjs/core/ace'),
  },
}))

ignitor.ace().catch(console.error)
