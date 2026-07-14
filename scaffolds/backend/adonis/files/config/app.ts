import { defineConfig } from '@adonisjs/core'

export default defineConfig({
  appKey: process.env.APP_KEY || '',
  http: {
    cookie: {},
    trustProxy: () => true,
  },
})
