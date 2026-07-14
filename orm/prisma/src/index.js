import { readFileSync, statSync, writeFileSync, mkdirSync } from 'fs'
import { join } from 'path'

export function createORMAdapter() {
  return {
    name: 'prisma',
    detect(dir) {
      return existsSync(join(dir, 'prisma', 'schema.prisma'))
    },
    supportedVersions: ['5.0', '5.22', '6.0'],
    defaultVersion: '6.0',
    kind: 'ORM',

    async generate(dir) {
      const { execFile } = await import('child_process')
      await execFile('npx', ['prisma', 'generate'], { cwd: dir })
    },

    async migrate(dir, name) {
      const { execFile } = await import('child_process')
      const args = ['prisma', 'migrate', 'dev', '--name', name || 'init']
      await execFile('npx', args, { cwd: dir })
    },

    async push(dir) {
      const { execFile } = await import('child_process')
      await execFile('npx', ['prisma', 'db', 'push'], { cwd: dir })
    },

    async studio(dir) {
      const { spawn } = await import('child_process')
      const proc = spawn('npx', ['prisma', 'studio'], { cwd: dir, stdio: 'inherit' })
      return new Promise((resolve, reject) => {
        proc.on('close', (code) => code === 0 ? resolve() : reject(new Error(`Studio exited with code ${code}`)))
      })
    },

    scaffold(name, options) {
      return scaffoldPrisma(name, options)
    }
  }
}

function existsSync(p) {
  try { return statSync(p).isFile() } catch { return false }
}

async function scaffoldPrisma(name, options) {
  const projectDir = join(options.dir || process.cwd(), name)
  mkdirSync(join(projectDir, 'prisma'), { recursive: true })
  mkdirSync(join(projectDir, 'src'), { recursive: true })

  writeFileSync(join(projectDir, 'prisma', 'schema.prisma'), `generator client {
  provider = "prisma-client-js"
}

datasource db {
  provider = "postgresql"
  url      = env("DATABASE_URL")
}

model User {
  id        String   @id @default(cuid())
  email     String   @unique
  name      String?
  createdAt DateTime @default(now())
  updatedAt DateTime @updatedAt
  posts     Post[]
}

model Post {
  id        String   @id @default(cuid())
  title     String
  content   String?
  published Boolean  @default(false)
  author    User     @relation(fields: [authorId], references: [id])
  authorId  String
  createdAt DateTime @default(now())
  updatedAt DateTime @updatedAt
}
`)

  writeFileSync(join(projectDir, '.env'), 'DATABASE_URL="postgresql://user:password@localhost:5432/mydb"\n')
  writeFileSync(join(projectDir, 'src', 'index.ts'), `import { PrismaClient } from '@prisma/client'

const prisma = new PrismaClient()

async function main() {
  const users = await prisma.user.findMany({ include: { posts: true } })
  console.log(JSON.stringify(users, null, 2))
}

main()
  .catch(console.error)
  .finally(() => prisma.\$disconnect())
`)
  writeFileSync(join(projectDir, 'README.md'), `# ${name}\n\nPrisma ORM\n\nnpx prisma generate\nnpx prisma migrate dev\n`)
}
