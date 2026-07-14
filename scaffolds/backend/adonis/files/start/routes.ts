import router from '@adonisjs/core/services/router'

router.get('/', async () => {
  return { message: 'Hello from {{ name }}', version: '{{ version }}' }
})

router.get('/health', async () => {
  return { status: 'ok', service: '{{ name }}' }
})
