import Hapi from '@hapi/hapi'
const init = async () => {
  const server = Hapi.server({ port: 3000 })
  server.route({ method: 'GET', path: '/', handler: (req, h) => 'Hello Hapi' })
  await server.start()
}
init()
