import { builder } from '../schema/builder.js'

builder.queryType({
  fields: (t) => ({
    hello: t.string({
      args: { name: t.arg.string({ required: true }) },
      resolve: (_, { name }) => `Hello, ${name}!`,
    }),
    version: t.string({
      resolve: () => '{{ version }}',
    }),
    info: t.string({
      resolve: () => '{{ name }} - {{ description }}',
    }),
  }),
})
