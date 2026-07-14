import { readFileSync, statSync, writeFileSync, mkdirSync } from 'fs'
import { join } from 'path'

export function createORMAdapter() {
  return {
    name: 'mikroorm',
    detect(dir) {
      try {
        const pkg = JSON.parse(readFileSync(join(dir, 'package.json'), 'utf-8'))
        return pkg?.dependencies?.['@mikro-orm/core'] || pkg?.devDependencies?.['@mikro-orm/core']
      } catch { return false }
    },
    supportedVersions: ['6.0', '6.3'],
    defaultVersion: '6.3',
    kind: 'ORM',

    scaffold(name, options) {
      return scaffoldMikroORM(name, options)
    }
  }
}

function existsSync(p) {
  try { return statSync(p).isFile() } catch { return false }
}

async function scaffoldMikroORM(name, options) {
  const projectDir = join(options.dir || process.cwd(), name)
  mkdirSync(join(projectDir, 'src', 'entities'), { recursive: true })

  writeFileSync(join(projectDir, 'package.json'), JSON.stringify({
    name, version: '1.0.0', private: true, type: 'module',
    scripts: { dev: 'tsx src/index.ts', build: 'tsc', 'schema:update': 'mikro-orm schema:update --run', 'schema:drop': 'mikro-orm schema:drop --run' },
    dependencies: {
      '@mikro-orm/core': '^6.3.0', '@mikro-orm/postgresql': '^6.3.0',
      '@mikro-orm/reflection': '^6.3.0', 'reflect-metadata': '^0.2.0'
    },
    devDependencies: { typescript: '^5.7.0', tsx: '^4.19.0', '@mikro-orm/cli': '^6.3.0' },
    mikroorm: { entities: ['./dist/entities'], entitiesTs: ['./src/entities'], dbName: name, host: 'localhost', port: 5432, type: 'postgresql' }
  }, null, 2))

  writeFileSync(join(projectDir, 'tsconfig.json'), JSON.stringify({
    compilerOptions: {
      target: 'ES2022', module: 'ESNext', moduleResolution: 'bundler',
      strict: true, skipLibCheck: true, noEmit: true, isolatedModules: true,
      experimentalDecorators: true, emitDecoratorMetadata: true
    },
    include: ['src']
  }, null, 2))

  writeFileSync(join(projectDir, 'src', 'entities', 'User.ts'), `import { Entity, PrimaryKey, Property, OneToMany, Collection } from '@mikro-orm/core'
import { Post } from './Post.js'

@Entity()
export class User {
  @PrimaryKey()
  id!: number

  @Property({ unique: true })
  email!: string

  @Property({ nullable: true })
  name?: string

  @OneToMany(() => Post, (post) => post.author)
  posts = new Collection<Post>(this)

  @Property({ onCreate: () => new Date() })
  createdAt = new Date()

  @Property({ onUpdate: () => new Date() })
  updatedAt = new Date()
}
`)

  writeFileSync(join(projectDir, 'src', 'entities', 'Post.ts'), `import { Entity, PrimaryKey, Property, ManyToOne } from '@mikro-orm/core'
import { User } from './User.js'

@Entity()
export class Post {
  @PrimaryKey()
  id!: number

  @Property()
  title!: string

  @Property({ nullable: true })
  content?: string

  @Property({ default: false })
  published = false

  @ManyToOne(() => User)
  author!: User

  @Property({ onCreate: () => new Date() })
  createdAt = new Date()

  @Property({ onUpdate: () => new Date() })
  updatedAt = new Date()
}
`)

  writeFileSync(join(projectDir, 'src', 'index.ts'), `import 'reflect-metadata'
import { MikroORM } from '@mikro-orm/core'
import { PostgreSqlDriver } from '@mikro-orm/postgresql'

const orm = await MikroORM.init({
  entities: ['./dist/entities'],
  dbName: process.env.DB_NAME || '${name}',
  host: process.env.DB_HOST || 'localhost',
  port: Number(process.env.DB_PORT || 5432),
  type: 'postgresql',
})

const users = await orm.em.find('User', {})
console.log(JSON.stringify(users, null, 2))

await orm.close()
`)

  writeFileSync(join(projectDir, '.env'), 'DB_HOST=localhost\nDB_PORT=5432\nDB_NAME=mydb\n')
  writeFileSync(join(projectDir, 'README.md'), `# ${name}\n\nMikroORM\n\nnpm run schema:update\n`)
}
