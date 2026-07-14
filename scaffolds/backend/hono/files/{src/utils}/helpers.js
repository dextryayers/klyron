export function formatResponse(data, status = 200) {
  return new Response(JSON.stringify(data), {
    status,
    headers: { 'Content-Type': 'application/json' },
  })
}

export function parseQuery(searchParams) {
  const params = {}
  for (const [key, value] of searchParams.entries()) {
    params[key] = value
  }
  return params
}

export function paginate(items, page = 1, limit = 10) {
  const start = (page - 1) * limit
  const end = start + limit
  return {
    data: items.slice(start, end),
    page,
    limit,
    total: items.length,
    totalPages: Math.ceil(items.length / limit),
  }
}
