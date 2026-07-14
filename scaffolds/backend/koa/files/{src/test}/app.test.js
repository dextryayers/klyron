import { describe, it, expect } from 'vitest'
import request from 'supertest'
import app from '../index.js'

describe('Koa App', () => {
  it('GET /health should return ok', async () => {
    const res = await request(app.callback()).get('/health')
    expect(res.status).toBe(200)
    expect(res.body).toMatchObject({ status: 'ok' })
  })

  it('GET /api should return message', async () => {
    const res = await request(app.callback()).get('/api')
    expect(res.status).toBe(200)
    expect(res.body).toHaveProperty('message')
    expect(res.body).toHaveProperty('version')
  })

  it('POST /api/echo should return body', async () => {
    const res = await request(app.callback())
      .post('/api/echo')
      .send({ test: true })
    expect(res.status).toBe(200)
    expect(res.body.received).toEqual({ test: true })
  })
})
