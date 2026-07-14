import Koa from 'koa'
import Router from '@koa/router'
import cors from 'koa-cors'
import helmet from 'koa-helmet'
import { koaBody } from 'koa-body'
import { apiRouter } from './routes/api.js'

const app = new Koa()
const router = new Router()

app.use(helmet())
app.use(cors())
app.use(koaBody())

router.get('/health', (ctx) => {
  ctx.body = { status: 'ok', service: '{{ name }}' }
})

app.use(router.routes())
app.use(router.allowedMethods())
app.use(apiRouter.routes())
app.use(apiRouter.allowedMethods())

const port = parseInt(process.env.PORT || '3000')
app.listen(port, () => {
  console.log(`{{ name }} running on port ${port}`)
})

export default app
