import { builder } from '../schema/builder.js'

const messages = []

const Message = builder.objectType('Message', {
  fields: (t) => ({
    id: t.exposeString('id'),
    content: t.exposeString('content'),
    author: t.exposeString('author'),
    createdAt: t.exposeString('createdAt'),
  }),
})

builder.mutationType({
  fields: (t) => ({
    createMessage: t.field({
      type: Message,
      args: {
        content: t.arg.string({ required: true }),
        author: t.arg.string({ required: true }),
      },
      resolve: (_, { content, author }) => {
        const message = {
          id: String(Date.now()),
          content,
          author,
          createdAt: new Date().toISOString(),
        }
        messages.push(message)
        return message
      },
    }),
    deleteMessage: t.field({
      type: 'Boolean',
      args: { id: t.arg.string({ required: true }) },
      resolve: (_, { id }) => {
        const index = messages.findIndex((m) => m.id === id)
        if (index === -1) return false
        messages.splice(index, 1)
        return true
      },
    }),
  }),
})
