import { test, expect } from 'vitest'
import app from '../index.js'

test('GET /health returns ok', async () => {
  const res = await app.fetch(new Request('http://localhost/health'))
  expect(res.status).toBe(200)
  const body = await res.json()
  expect(body).toMatchObject({ status: 'ok' })
})

test('GET /api returns hello message', async () => {
  const res = await app.fetch(new Request('http://localhost/api'))
  expect(res.status).toBe(200)
  const body = await res.json()
  expect(body).toHaveProperty('message')
  expect(body).toHaveProperty('version')
})

test('POST /api/echo returns received data', async () => {
  const payload = { message: 'test' }
  const res = await app.fetch(
    new Request('http://localhost/api/echo', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
    }),
  )
  expect(res.status).toBe(200)
  const body = await res.json()
  expect(body.received).toEqual(payload)
})
