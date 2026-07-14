export function createResponse(data: unknown, status = 200) {
  return { data, status }
}
