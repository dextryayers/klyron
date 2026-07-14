import { readFileSync, statSync, writeFileSync, mkdirSync } from 'fs'
import { join } from 'path'

export function createAdapter() {
  return {
    name: 'fastify',
    detect(dir) {
      try {
        const pkg = readPackageJson(dir)
        return pkg?.dependencies?.fastify || pkg?.devDependencies?.fastify
      } catch { return false }
    },
    supportedVersions: ['4.28', '5.0', '5.2'],
    defaultVersion: '5.2',
    kind: 'Backend',

    async dev(dir, port) {
      const { spawn } = await import('child_process')
      const hasTS = existsSync(join(dir, 'tsconfig.json'))
      const args = hasTS ? ['tsx', 'watch', 'src/index.ts'] : ['node', '--watch', 'src/index.js']
      const proc = spawn('npx', args, { cwd: dir, env: { ...process.env, PORT: String(port || 3000) }, stdio: 'inherit' })
      return new Promise((resolve, reject) => {
        proc.on('close', (code) => code === 0 ? resolve() : reject(new Error(`Dev server exited with code ${code}`)))
      })
    },

    async build(dir) {
      const { execFile } = await import('child_process')
      if (existsSync(join(dir, 'tsconfig.json'))) {
        await execFile('npx', ['tsc'], { cwd: dir })
      }
    },

    async test(dir) {
      const { execFile } = await import('child_process')
      await execFile('npx', ['vitest', 'run'], { cwd: dir })
    },

    async lint(dir) {
      const { execFile } = await import('child_process')
      await execFile('npx', ['eslint', '.'], { cwd: dir })
    },

    async format(dir, writeMode) {
      const { execFile } = await import('child_process')
      await execFile('npx', writeMode ? ['prettier', '--write', '.'] : ['prettier', '--check', '.'], { cwd: dir })
    },

    scaffold(name, options) {
      return scaffoldFastify(name, options)
    }
  }
}

function readPackageJson(dir) {
  try {
    return JSON.parse(readFileSync(join(dir, 'package.json'), 'utf-8'))
  } catch { return null }
}

function existsSync(p) {
  try { return statSync(p).isFile() } catch { return false }
}

async function scaffoldFastify(name, options) {
  const projectDir = join(options.dir || process.cwd(), name)
  mkdirSync(join(projectDir, 'src', 'routes'), { recursive: true })
  mkdirSync(join(projectDir, 'src', 'plugins'), { recursive: true })

  writeFileSync(join(projectDir, 'package.json'), JSON.stringify({
    name, version: '1.0.0', private: true, type: 'module',
    scripts: { dev: 'node --watch src/index.js', start: 'node src/index.js', test: 'vitest run', lint: 'eslint .', format: 'prettier --write .' },
    dependencies: { fastify: '^5.2.0', '@fastify/cors': '^10.0.0', '@fastify/env': '^5.0.0' },
    devDependencies: { vitest: '^3.0.0', eslint: '^9.20.0', prettier: '^3.5.0' }
  }, null, 2))

  writeFileSync(join(projectDir, 'src', 'index.js'), `import Fastify from 'fastify'
import cors from '@fastify/cors'
import routes from './routes/index.js'

const app = Fastify({ logger: true })

await app.register(cors)
await app.register(routes)

const start = async () => {
  try {
    await app.listen({ port: process.env.PORT || 3000, host: '0.0.0.0' })
    console.log(\`${name} running on http://localhost:\${process.env.PORT || 3000}\`)
  } catch (err) {
    app.log.error(err)
    process.exit(1)
  }
}

start()
`)

  writeFileSync(join(projectDir, 'src', 'routes', 'index.js'), `export default async function (fastify, opts) {
  fastify.get('/', async (request, reply) => {
    return { message: 'Hello World' }
  })

  fastify.get('/health', async (request, reply) => {
    return { status: 'ok', timestamp: new Date().toISOString() }
  })
}
`)

  writeFileSync(join(projectDir, 'src', 'plugins', 'README.js'), `// Fastify plugins go here
`)

  writeFileSync(join(projectDir, '.gitignore'), 'node_modules\n.DS_Store\n')
  writeFileSync(join(projectDir, '.prettierrc'), JSON.stringify({ semi: true, singleQuote: true, tabWidth: 2, trailingComma: 'es5', printWidth: 100 }))
  writeFileSync(join(projectDir, 'README.md'), `# ${name}\n\nFastify API\n\nnpm run dev\n`)
}
