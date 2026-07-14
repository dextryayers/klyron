import { readFileSync, statSync, writeFileSync, mkdirSync } from 'fs'
import { join } from 'path'

export function createORMAdapter() {
  return {
    name: 'typeorm',
    detect(dir) {
      try {
        const pkg = JSON.parse(readFileSync(join(dir, 'package.json'), 'utf-8'))
        const hasTypeorm = pkg?.dependencies?.typeorm || pkg?.devDependencies?.typeorm
        const hasDataSource = existsSync(join(dir, 'data-source.ts')) || existsSync(join(dir, 'typeorm.config.ts'))
        return hasTypeorm || hasDataSource
      } catch { return false }
    },
    supportedVersions: ['0.3'],
    defaultVersion: '0.3',
    kind: 'ORM',

    scaffold(name, options) {
      return scaffoldTypeORM(name, options)
    }
  }
}

function existsSync(p) {
  try { return statSync(p).isFile() } catch { return false }
}

async function scaffoldTypeORM(name, options) {
  const projectDir = join(options.dir || process.cwd(), name)
  mkdirSync(join(projectDir, 'src', 'entity'), { recursive: true })

  writeFileSync(join(projectDir, 'package.json'), JSON.stringify({
    name, version: '1.0.0', private: true, type: 'module',
    scripts: { dev: 'tsx src/index.ts', build: 'tsc', 'schema:sync': 'typeorm schema:sync', 'migration:run': 'typeorm migration:run', 'migration:generate': 'typeorm migration:generate' },
    dependencies: { typeorm: '^0.3.20', 'reflect-metadata': '^0.2.0', pg: '^8.13.0' },
    devDependencies: { typescript: '^5.7.0', tsx: '^4.19.0', '@types/node': '^22.13.0' }
  }, null, 2))

  writeFileSync(join(projectDir, 'tsconfig.json'), JSON.stringify({
    compilerOptions: {
      target: 'ES2022', module: 'ESNext', moduleResolution: 'bundler',
      strict: true, skipLibCheck: true, noEmit: true, isolatedModules: true,
      experimentalDecorators: true, emitDecoratorMetadata: true
    },
    include: ['src']
  }, null, 2))

  writeFileSync(join(projectDir, 'src', 'data-source.ts'), `import 'reflect-metadata'
import { DataSource } from 'typeorm'
import { User } from './entity/User.js'
import { Post } from './entity/Post.js'

export const AppDataSource = new DataSource({
  type: 'postgres',
  host: process.env.DB_HOST || 'localhost',
  port: Number(process.env.DB_PORT || 5432),
  username: process.env.DB_USER || 'postgres',
  password: process.env.DB_PASSWORD || 'postgres',
  database: process.env.DB_NAME || '${name}',
  synchronize: true,
  logging: false,
  entities: [User, Post],
  migrations: [],
  subscribers: [],
})
`)

  writeFileSync(join(projectDir, 'src', 'entity', 'User.ts'), `import { Entity, PrimaryGeneratedColumn, Column, OneToMany } from 'typeorm'
import { Post } from './Post.js'

@Entity()
export class User {
  @PrimaryGeneratedColumn()
  id!: number

  @Column({ unique: true })
  email!: string

  @Column({ nullable: true })
  name?: string

  @OneToMany(() => Post, (post) => post.author)
  posts!: Post[]

  @Column({ type: 'timestamp', default: () => 'CURRENT_TIMESTAMP' })
  createdAt!: Date

  @Column({ type: 'timestamp', default: () => 'CURRENT_TIMESTAMP', onUpdate: 'CURRENT_TIMESTAMP' })
  updatedAt!: Date
}
`)

  writeFileSync(join(projectDir, 'src', 'entity', 'Post.ts'), `import { Entity, PrimaryGeneratedColumn, Column, ManyToOne, JoinColumn } from 'typeorm'
import { User } from './User.js'

@Entity()
export class Post {
  @PrimaryGeneratedColumn()
  id!: number

  @Column()
  title!: string

  @Column({ nullable: true })
  content?: string

  @Column({ default: false })
  published!: boolean

  @ManyToOne(() => User, (user) => user.posts)
  @JoinColumn({ name: 'authorId' })
  author!: User

  @Column({ type: 'timestamp', default: () => 'CURRENT_TIMESTAMP' })
  createdAt!: Date

  @Column({ type: 'timestamp', default: () => 'CURRENT_TIMESTAMP', onUpdate: 'CURRENT_TIMESTAMP' })
  updatedAt!: Date
}
`)

  writeFileSync(join(projectDir, 'src', 'index.ts'), `import 'reflect-metadata'
import { AppDataSource } from './data-source.js'

await AppDataSource.initialize()
console.log('Database connected')

const users = await AppDataSource.manager.find('User', { relations: ['posts'] })
console.log(JSON.stringify(users, null, 2))

await AppDataSource.destroy()
`)

  writeFileSync(join(projectDir, '.env'), 'DB_HOST=localhost\nDB_PORT=5432\nDB_USER=postgres\nDB_PASSWORD=postgres\nDB_NAME=mydb\n')
  writeFileSync(join(projectDir, 'README.md'), `# ${name}\n\nTypeORM\n\nnpm run schema:sync\n`)
}
