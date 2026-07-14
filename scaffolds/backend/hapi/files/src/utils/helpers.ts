export function successResponse(data: unknown) {
  return { success: true, data }
}

export function errorResponse(message: string, code = 400) {
  return { success: false, error: message, code }
}
