import { test, expect } from 'vitest'
import { buildApp } from '../src/index.js'

test('GET /health returns ok', async () => {
  const app = await buildApp()
  const response = await app.inject({ method: 'GET', url: '/health' })
  expect(response.statusCode).toBe(200)
  expect(response.json()).toMatchObject({ status: 'ok' })
})

test('GET /api returns hello message', async () => {
  const app = await buildApp()
  const response = await app.inject({ method: 'GET', url: '/api' })
  expect(response.statusCode).toBe(200)
  expect(response.json()).toHaveProperty('message')
})

test('POST /api/echo returns received payload', async () => {
  const app = await buildApp()
  const payload = { message: 'test' }
  const response = await app.inject({
    method: 'POST',
    url: '/api/echo',
    payload,
  })
  expect(response.statusCode).toBe(200)
  expect(response.json()).toEqual({ received: payload })
})
