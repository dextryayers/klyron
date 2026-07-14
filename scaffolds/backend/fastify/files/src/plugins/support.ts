import fp from 'fastify-plugin'
import type { FastifyInstance } from 'fastify'

export default fp(async function (fastify: FastifyInstance) {
  fastify.decorate('support', { version: '{{ version }}' })
})

declare module 'fastify' {
  interface FastifyInstance {
    support: { version: string }
  }
}
