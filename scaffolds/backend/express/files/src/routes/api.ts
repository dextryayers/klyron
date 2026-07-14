import { Router, Request, Response } from 'express'

export const apiRouter = Router()

apiRouter.get('/', (_req: Request, res: Response) => {
  res.json({ message: 'Hello from {{ name }}', version: '{{ version }}' })
})

apiRouter.post('/echo', (req: Request, res: Response) => {
  res.json({ received: req.body })
})
