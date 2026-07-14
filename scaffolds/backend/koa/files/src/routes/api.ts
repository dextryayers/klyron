import Router from '@koa/router'

export const apiRouter = new Router({ prefix: '/api' })

apiRouter.get('/', (ctx) => {
  ctx.body = { message: 'Hello from {{ name }}', version: '{{ version }}' }
})

apiRouter.post('/echo', (ctx) => {
  ctx.body = { received: ctx.request.body }
})
