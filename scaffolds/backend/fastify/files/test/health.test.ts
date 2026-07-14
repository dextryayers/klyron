import { test, expect } from 'vitest'

test('health endpoint', async () => {
  const response = { status: 'ok', service: '{{ name }}' }
  expect(response.status).toBe('ok')
})
