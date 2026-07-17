import { Elysia } from 'elysia'
const app = new Elysia().get('/', () => 'Hello Elysia')
app.listen(3000)
