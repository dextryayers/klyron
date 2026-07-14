import Fastify from 'fastify'
import cors from '@fastify/cors'
import swagger from '@fastify/swagger'
import swaggerUi from '@fastify/swagger-ui'
import { apiRoutes } from './routes/api.js'

const server = Fastify({ logger: true })

await server.register(cors)
await server.register(swagger, {
  openapi: {
    info: { title: '{{ name }}', version: '{{ version }}', description: '{{ description }}' },
  },
})
await server.register(swaggerUi, { routePrefix: '/docs' })

await server.register(apiRoutes, { prefix: '/api' })

server.get('/health', async () => ({ status: 'ok', service: '{{ name }}' }))

const start = async () => {
  try {
    await server.listen({ port: parseInt(process.env.PORT || '3000') })
  } catch (err) {
    server.log.error(err)
    process.exit(1)
  }
}

start()

export default server
