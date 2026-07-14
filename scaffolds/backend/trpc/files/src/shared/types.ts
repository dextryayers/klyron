import { z } from 'zod'

export const HelloSchema = z.object({
  name: z.string().min(1),
})

export type HelloInput = z.infer<typeof HelloSchema>
