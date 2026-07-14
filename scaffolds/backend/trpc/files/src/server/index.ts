import express from 'express'
import { createExpressMiddleware } from '@trpc/server/adapters/express'
import { appRouter } from './routers/index.js'

const app = express()

app.use('/trpc', createExpressMiddleware({ router: appRouter }))

app.get('/health', (_req, res) => {
  res.json({ status: 'ok', service: '{{ name }}' })
})

app.listen(3000, () => {
  console.log(`{{ name }} running on port 3000`)
})

export type { AppRouter } from './routers/index.js'
