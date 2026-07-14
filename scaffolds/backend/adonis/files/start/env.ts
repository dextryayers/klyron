import Env from '@adonisjs/core/services/env'

export default await Env.create(new URL('../', import.meta.url), {
  PORT: Env.schema.number(),
  HOST: Env.schema.string(),
  NODE_ENV: Env.schema.enum(['development', 'production', 'test'] as const),
  APP_KEY: Env.schema.string(),
  DB_CONNECTION: Env.schema.enum(['pg', 'mysql', 'sqlite'] as const),
})
