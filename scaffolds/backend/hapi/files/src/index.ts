import Hapi from '@hapi/hapi'
import { apiPlugin } from './routes/api.js'

const init = async () => {
  const server = Hapi.server({
    port: parseInt(process.env.PORT || '3000'),
    host: 'localhost',
  })

  await server.register(apiPlugin)

  server.route({
    method: 'GET',
    path: '/health',
    handler: () => ({ status: 'ok', service: '{{ name }}' }),
  })

  await server.start()
  console.log(`{{ name }} running on ${server.info.uri}`)
}

process.on('unhandledRejection', (err) => {
  console.error(err)
  process.exit(1)
})

init()

export default init
