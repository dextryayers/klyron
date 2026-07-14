import { readFileSync, statSync, writeFileSync, mkdirSync } from 'fs'
import { join } from 'path'

export function createORMAdapter() {
  return {
    name: 'knex',
    detect(dir) {
      try {
        const pkg = JSON.parse(readFileSync(join(dir, 'package.json'), 'utf-8'))
        const hasKnex = pkg?.dependencies?.knex || pkg?.devDependencies?.knex
        const hasKnexfile = existsSync(join(dir, 'knexfile.ts')) || existsSync(join(dir, 'knexfile.js'))
        return hasKnex || hasKnexfile
      } catch { return false }
    },
    supportedVersions: ['3.1'],
    defaultVersion: '3.1',
    kind: 'ORM',

    scaffold(name, options) {
      return scaffoldKnex(name, options)
    }
  }
}

function existsSync(p) {
  try { return statSync(p).isFile() } catch { return false }
}

async function scaffoldKnex(name, options) {
  const projectDir = join(options.dir || process.cwd(), name)
  mkdirSync(join(projectDir, 'src'), { recursive: true })
  mkdirSync(join(projectDir, 'migrations'), { recursive: true })
  mkdirSync(join(projectDir, 'seeds'), { recursive: true })

  writeFileSync(join(projectDir, 'package.json'), JSON.stringify({
    name, version: '1.0.0', private: true, type: 'module',
    scripts: { dev: 'node src/index.js', 'migrate:latest': 'knex migrate:latest', 'migrate:rollback': 'knex migrate:rollback', 'seed:run': 'knex seed:run' },
    dependencies: { knex: '^3.1.0', pg: '^8.13.0' }
  }, null, 2))

  writeFileSync(join(projectDir, 'knexfile.js'), `export default {
  development: {
    client: 'pg',
    connection: process.env.DATABASE_URL || 'postgresql://user:password@localhost:5432/${name}',
    migrations: { directory: './migrations' },
    seeds: { directory: './seeds' }
  }
}
`)

  writeFileSync(join(projectDir, 'migrations', '20240101000000_create_users.js'), `export function up(knex) {
  return knex.schema.createTable('users', (table) => {
    table.increments('id').primary()
    table.string('email').unique().notNullable()
    table.string('name')
    table.timestamps(true, true)
  })
}

export function down(knex) {
  return knex.schema.dropTable('users')
}
`)

  writeFileSync(join(projectDir, 'migrations', '20240101000001_create_posts.js'), `export function up(knex) {
  return knex.schema.createTable('posts', (table) => {
    table.increments('id').primary()
    table.string('title').notNullable()
    table.text('content')
    table.boolean('published').defaultTo(false)
    table.integer('author_id').references('id').inTable('users')
    table.timestamps(true, true)
  })
}

export function down(knex) {
  return knex.schema.dropTable('posts')
}
`)

  writeFileSync(join(projectDir, 'seeds', '01_users.js'), `export async function seed(knex) {
  await knex('users').del()
  await knex('users').insert([
    { email: 'user1@example.com', name: 'User One' },
    { email: 'user2@example.com', name: 'User Two' }
  ])
}
`)

  writeFileSync(join(projectDir, 'src', 'index.js'), `import knex from 'knex'
import config from '../knexfile.js'

const db = knex(config.development)

const users = await db('users').select('*')
console.log(JSON.stringify(users, null, 2))

await db.destroy()
`)

  writeFileSync(join(projectDir, '.env'), 'DATABASE_URL="postgresql://user:password@localhost:5432/mydb"\n')
  writeFileSync(join(projectDir, 'README.md'), `# ${name}\n\nKnex.js Query Builder\n\nnpm run migrate:latest\n`)
}
