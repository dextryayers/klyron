import { createYoga } from 'graphql-yoga'
import { createServer } from 'node:http'
import { builder } from './schema/builder.js'
import './schema/query.js'

const schema = builder.toSchema()

const yoga = createYoga({ schema })

const server = createServer(yoga)

server.listen(4000, () => {
  console.log(`{{ name }} GraphQL server running on http://localhost:4000/graphql`)
})
