import { Hono } from 'hono'
import { z } from 'zod'
import { zValidator } from '@hono/zod-validator'

export const apiRoutes = new Hono()

apiRoutes.get('/', (c) => c.json({
  message: 'Hello from {{ name }}',
  version: '{{ version }}',
}))

const echoSchema = z.object({
  message: z.string(),
})

apiRoutes.post('/echo', zValidator('json', echoSchema), (c) => {
  const data = c.req.valid('json')
  return c.json({ received: data })
})
