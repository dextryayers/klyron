import express from 'express'
import cors from 'cors'
import helmet from 'helmet'
import morgan from 'morgan'
import { apiRouter } from './routes/api'
import { errorHandler } from './middleware/errorHandler'

const app = express()
const port = process.env.PORT || 3000

app.use(helmet())
app.use(cors())
app.use(morgan('dev'))
app.use(express.json())

app.get('/health', (_req, res) => {
  res.json({ status: 'ok', service: '{{ name }}' })
})

app.use('/api', apiRouter)
app.use(errorHandler)

app.listen(port, () => {
  console.log(`{{ name }} running on port ${port}`)
})

export default app
