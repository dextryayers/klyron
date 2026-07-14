import 'reflect-metadata'
import { Ignitor } from '@adonisjs/core'
import { defineConfig } from '@adonisjs/core/assembler'

const ignitor = Ignitor.makeApp(defineConfig({
  commands: () => import('@adonisjs/core/commands'),
  http: () => import('./start/routes.js'),
}))

ignitor.httpServer().catch(console.error)
