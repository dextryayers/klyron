import type { APIRoute } from 'astro'

export const GET: APIRoute = async () => {
  return new Response(JSON.stringify({
    message: 'Hello from {{ name }}',
    version: '{{ version }}',
  }), { status: 200 })
}
