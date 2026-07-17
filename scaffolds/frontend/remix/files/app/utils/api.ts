import { json } from '@remix-run/node'

export function apiResponse(data: unknown, status = 200) {
  return json(data, { status })
}

export function apiError(message: string, status = 400) {
  return json({ error: message }, { status })
}
