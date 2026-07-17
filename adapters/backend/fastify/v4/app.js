import Fastify from 'fastify'
const app = Fastify({ logger: true })
app.get('/', async (req, reply) => ({ hello: 'world' }))
app.listen({ port: 3000 })
