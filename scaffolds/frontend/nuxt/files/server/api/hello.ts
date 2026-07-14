export default defineEventHandler(() => {
  return {
    message: 'Hello from {{ name }}',
    version: '{{ version }}',
  }
})
