import Hapi from '@hapi/hapi'

export const loggerPlugin: Hapi.Plugin<undefined> = {
  name: 'logger',
  register: async (server: Hapi.Server) => {
    server.events.on('response', (request) => {
      console.log(`${request.method.toUpperCase()} ${request.path} ${(request as any).response?.statusCode}`)
    })
  },
}
