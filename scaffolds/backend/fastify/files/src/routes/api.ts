import { FastifyInstance } from 'fastify'

export async function apiRoutes(app: FastifyInstance) {
  app.get('/', async () => ({
    message: 'Hello from {{ name }}',
    version: '{{ version }}',
  }))

  app.post<{ Body: Record<string, unknown> }>('/echo', async (request) => {
    return { received: request.body }
  })
}
