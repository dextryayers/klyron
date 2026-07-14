import { Hono } from 'hono'
import { cors } from 'hono/cors'
import { logger } from 'hono/logger'
import { apiRoutes } from './routes/api'

const app = new Hono()

app.use('*', cors())
app.use('*', logger())

app.get('/health', (c) => c.json({ status: 'ok', service: '{{ name }}' }))

app.route('/api', apiRoutes)

export default app
