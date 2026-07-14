import { publicProcedure, router } from '../trpc'
import { HelloSchema } from '../../shared/types.js'

export const helloRouter = router({
  greet: publicProcedure
    .input(HelloSchema)
    .query(({ input }) => ({
      message: `Hello, ${input.name}!`,
      service: '{{ name }}',
    })),
  version: publicProcedure
    .query(() => ({
      version: '{{ version }}',
    })),
})
