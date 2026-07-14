import { readFileSync, statSync, writeFileSync, mkdirSync } from 'fs'
import { join } from 'path'

export function createORMAdapter() {
  return {
    name: 'kysely',
    detect(dir) {
      try {
        const pkg = JSON.parse(readFileSync(join(dir, 'package.json'), 'utf-8'))
        return pkg?.dependencies?.kysely || pkg?.devDependencies?.kysely
      } catch { return false }
    },
    supportedVersions: ['0.27'],
    defaultVersion: '0.27',
    kind: 'ORM',

    scaffold(name, options) {
      return scaffoldKysely(name, options)
    }
  }
}

function existsSync(p) {
  try { return statSync(p).isFile() } catch { return false }
}

async function scaffoldKysely(name, options) {
  const projectDir = join(options.dir || process.cwd(), name)
  mkdirSync(join(projectDir, 'src', 'db'), { recursive: true })

  writeFileSync(join(projectDir, 'package.json'), JSON.stringify({
    name, version: '1.0.0', private: true, type: 'module',
    scripts: { dev: 'tsx src/index.ts', build: 'tsc' },
    dependencies: { kysely: '^0.27.0', pg: '^8.13.0' },
    devDependencies: { typescript: '^5.7.0', tsx: '^4.19.0' }
  }, null, 2))

  writeFileSync(join(projectDir, 'tsconfig.json'), JSON.stringify({
    compilerOptions: { target: 'ES2022', module: 'ESNext', moduleResolution: 'bundler', strict: true, skipLibCheck: true, noEmit: true, isolatedModules: true },
    include: ['src']
  }, null, 2))

  writeFileSync(join(projectDir, 'src', 'db', 'types.ts'), `import { Generated, ColumnType } from 'kysely'

export interface Database {
  users: UsersTable
  posts: PostsTable
}

export interface UsersTable {
  id: Generated<number>
  email: string
  name: string | null
  created_at: ColumnType<Date, string | undefined, never>
  updated_at: ColumnType<Date, string | undefined, never>
}

export interface PostsTable {
  id: Generated<number>
  title: string
  content: string | null
  published: boolean
  author_id: number | null
  created_at: ColumnType<Date, string | undefined, never>
  updated_at: ColumnType<Date, string | undefined, never>
}
`)

  writeFileSync(join(projectDir, 'src', 'db', 'index.ts'), `import { Kysely, PostgresDialect } from 'kysely'
import pg from 'pg'
import type { Database } from './types.js'

const dialect = new PostgresDialect({
  pool: new pg.Pool({ connectionString: process.env.DATABASE_URL })
})

export const db = new Kysely<Database>({ dialect })
`)

  writeFileSync(join(projectDir, 'src', 'index.ts'), `import { db } from './db/index.js'

const users = await db.selectFrom('users').selectAll().execute()
console.log(JSON.stringify(users, null, 2))
`)

  writeFileSync(join(projectDir, '.env'), 'DATABASE_URL="postgresql://user:password@localhost:5432/mydb"\n')
  writeFileSync(join(projectDir, 'README.md'), `# ${name}\n\nKysely SQL Query Builder\n\nnpm run dev\n`)
}
