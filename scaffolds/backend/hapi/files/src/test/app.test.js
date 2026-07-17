import { describe, it, expect, beforeAll, afterAll } from 'vitest'
import Hapi from '@hapi/hapi'

let server

beforeAll(async () => {
  server = Hapi.server({ port: 0, host: 'localhost' })
  server.route({
    method: 'GET',
    path: '/health',
    handler: () => ({ status: 'ok', service: '{{ name }}' }),
  })
  await server.start()
})

afterAll(async () => {
  await server.stop()
})

describe('Health Check', () => {
  it('should return status ok', async () => {
    const res = await server.inject({ method: 'GET', url: '/health' })
    expect(res.statusCode).toBe(200)
    expect(res.result).toMatchObject({ status: 'ok' })
  })
})
