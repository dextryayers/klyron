import compose from 'koa-compose'

export function requestTimer() {
  return async (ctx, next) => {
    const start = Date.now()
    await next()
    const ms = Date.now() - start
    ctx.set('X-Response-Time', `${ms}ms`)
  }
}

export function requestLogger() {
  return async (ctx, next) => {
    const start = Date.now()
    await next()
    const ms = Date.now() - start
    console.log(`${ctx.method} ${ctx.url} - ${ms}ms`)
  }
}

export function composeMiddleware(...middleware) {
  return compose(middleware)
}
