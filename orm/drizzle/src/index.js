import { readFileSync, statSync, writeFileSync, mkdirSync } from 'fs'
import { join } from 'path'

export function createORMAdapter() {
  return {
    name: 'drizzle',
    detect(dir) {
      try {
        const pkg = JSON.parse(readFileSync(join(dir, 'package.json'), 'utf-8'))
        const hasDrizzle = pkg?.dependencies?.['drizzle-orm'] || pkg?.devDependencies?.['drizzle-orm']
        const hasConfig = existsSync(join(dir, 'drizzle.config.ts')) || existsSync(join(dir, 'drizzle.config.js'))
        return hasDrizzle || hasConfig
      } catch { return false }
    },
    supportedVersions: ['0.38', '0.40'],
    defaultVersion: '0.40',
    kind: 'ORM',

    scaffold(name, options) {
      return scaffoldDrizzle(name, options)
    }
  }
}

function existsSync(p) {
  try { return statSync(p).isFile() } catch { return false }
}

async function scaffoldDrizzle(name, options) {
  const projectDir = join(options.dir || process.cwd(), name)
  mkdirSync(join(projectDir, 'src', 'db', 'schema'), { recursive: true })

  writeFileSync(join(projectDir, 'package.json'), JSON.stringify({
    name, version: '1.0.0', private: true, type: 'module',
    scripts: { dev: 'tsx src/index.ts', build: 'tsc', 'db:push': 'drizzle-kit push', 'db:generate': 'drizzle-kit generate', 'db:migrate': 'drizzle-kit migrate', 'db:studio': 'drizzle-kit studio' },
    dependencies: { 'drizzle-orm': '^0.40.0', 'postgres': '^3.4.0' },
    devDependencies: { 'drizzle-kit': '^0.30.0', typescript: '^5.7.0', tsx: '^4.19.0' }
  }, null, 2))

  writeFileSync(join(projectDir, 'tsconfig.json'), JSON.stringify({
    compilerOptions: { target: 'ES2022', module: 'ESNext', moduleResolution: 'bundler', strict: true, skipLibCheck: true, noEmit: true, isolatedModules: true },
    include: ['src']
  }, null, 2))

  writeFileSync(join(projectDir, 'drizzle.config.ts'), `import { defineConfig } from 'drizzle-kit'

export default defineConfig({
  schema: './src/db/schema/index.ts',
  out: './drizzle',
  dialect: 'postgresql',
  dbCredentials: { url: process.env.DATABASE_URL! },
})
`)

  writeFileSync(join(projectDir, 'src', 'db', 'schema', 'index.ts'), `import { pgTable, serial, text, varchar, timestamp, boolean } from 'drizzle-orm/pg-core'

export const users = pgTable('users', {
  id: serial('id').primaryKey(),
  email: varchar('email', { length: 255 }).notNull().unique(),
  name: text('name'),
  createdAt: timestamp('created_at').defaultNow().notNull(),
  updatedAt: timestamp('updated_at').defaultNow().notNull(),
})

export const posts = pgTable('posts', {
  id: serial('id').primaryKey(),
  title: text('title').notNull(),
  content: text('content'),
  published: boolean('published').default(false),
  authorId: serial('author_id').references(() => users.id),
  createdAt: timestamp('created_at').defaultNow().notNull(),
  updatedAt: timestamp('updated_at').defaultNow().notNull(),
})
`)

  writeFileSync(join(projectDir, 'src', 'db', 'index.ts'), `import { drizzle } from 'drizzle-orm/postgres-js'
import postgres from 'postgres'
import * as schema from './schema/index.js'

const connectionString = process.env.DATABASE_URL!
const client = postgres(connectionString)
export const db = drizzle(client, { schema })
`)

  writeFileSync(join(projectDir, 'src', 'index.ts'), `import { db } from './db/index.js'
import { users } from './db/schema/index.js'

const allUsers = await db.select().from(users)
console.log(JSON.stringify(allUsers, null, 2))
`)

  writeFileSync(join(projectDir, '.env'), 'DATABASE_URL="postgresql://user:password@localhost:5432/mydb"\n')
  writeFileSync(join(projectDir, 'README.md'), `# ${name}\n\nDrizzle ORM\n\nnpm run db:generate\nnpm run db:migrate\n`)
}
