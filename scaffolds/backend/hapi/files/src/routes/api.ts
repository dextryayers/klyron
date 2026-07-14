import Hapi from '@hapi/hapi'
import Joi from 'joi'

export const apiPlugin: Hapi.Plugin<undefined> = {
  name: 'api',
  register: async (server: Hapi.Server) => {
    server.route([
      {
        method: 'GET',
        path: '/api',
        handler: () => ({
          message: 'Hello from {{ name }}',
          version: '{{ version }}',
        }),
      },
      {
        method: 'POST',
        path: '/api/echo',
        options: {
          validate: {
            payload: Joi.object({
              message: Joi.string().required(),
            }),
          },
        },
        handler: (request) => ({ received: request.payload }),
      },
    ])
  },
}
