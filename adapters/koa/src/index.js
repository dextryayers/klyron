import { readFileSync, statSync, writeFileSync, mkdirSync } from 'fs'
import { join } from 'path'

export function createAdapter() {
  return {
    name: 'koa',
    detect(dir) {
      try {
        const pkg = readPackageJson(dir)
        return pkg?.dependencies?.koa || pkg?.devDependencies?.koa
      } catch { return false }
    },
    supportedVersions: ['2.15'],
    defaultVersion: '2.15',
    kind: 'Backend',

    async dev(dir, port) {
      const { spawn } = await import('child_process')
      const proc = spawn('npx', ['nodemon', 'src/index.js'], { cwd: dir, env: { ...process.env, PORT: String(port || 3000) }, stdio: 'inherit' })
      return new Promise((resolve, reject) => {
        proc.on('close', (code) => code === 0 ? resolve() : reject(new Error(`Dev server exited with code ${code}`)))
      })
    },

    async build() {},

    async test(dir) {
      const { execFile } = await import('child_process')
      await execFile('npx', ['jest'], { cwd: dir })
    },

    async lint(dir) {
      const { execFile } = await import('child_process')
      await execFile('npx', ['eslint', '.'], { cwd: dir })
    },

    async format(dir, write) {
      const { execFile } = await import('child_process')
      await execFile('npx', write ? ['prettier', '--write', '.'] : ['prettier', '--check', '.'], { cwd: dir })
    },

    scaffold(name, options) {
      return scaffoldKoa(name, options)
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

async function scaffoldKoa(name, options) {
  const projectDir = join(options.dir || process.cwd(), name)
  mkdirSync(join(projectDir, 'src', 'middleware'), { recursive: true })
  mkdirSync(join(projectDir, 'src', 'routes'), { recursive: true })

  writeFileSync(join(projectDir, 'package.json'), JSON.stringify({
    name, version: '1.0.0', private: true, type: 'module',
    scripts: {
      dev: 'nodemon src/index.js',
      start: 'node src/index.js',
      test: 'jest',
      lint: 'eslint .',
      format: 'prettier --write .'
    },
    dependencies: { koa: '^2.15.0', '@koa/router': '^12.0.0', koa-body: '^6.0.0' },
    devDependencies: { jest: '^30.0.0', eslint: '^9.20.0', prettier: '^3.5.0', nodemon: '^3.1.0' }
  }, null, 2))

  writeFileSync(join(projectDir, 'src', 'index.js'), `import Koa from 'koa'
import Router from '@koa/router'
import { koaBody } from 'koa-body'
import routes from './routes/index.js'
import { errorHandler } from './middleware/error.js'

const app = new Koa()
const port = process.env.PORT || 3000

app.use(errorHandler)
app.use(koaBody())
app.use(routes.routes())
app.use(routes.allowedMethods())

app.listen(port, () => console.log(\`Koa server running on http://localhost:\${port}\`))
`)

  writeFileSync(join(projectDir, 'src', 'routes', 'index.js'), `import Router from '@koa/router'

const router = new Router()

router.get('/', (ctx) => {
  ctx.body = { message: 'Hello World' }
})

router.get('/health', (ctx) => {
  ctx.body = { status: 'ok', timestamp: new Date().toISOString() }
})

export default router
`)

  writeFileSync(join(projectDir, 'src', 'middleware', 'error.js'), `export async function errorHandler(ctx, next) {
  try {
    await next()
  } catch (err) {
    ctx.status = err.status || 500
    ctx.body = { error: err.message || 'Internal Server Error' }
  }
}
`)

  writeFileSync(join(projectDir, '.gitignore'), 'node_modules\n.DS_Store\n')
  writeFileSync(join(projectDir, '.prettierrc'), JSON.stringify({ semi: true, singleQuote: true, tabWidth: 2, trailingComma: 'es5', printWidth: 100 }))
  writeFileSync(join(projectDir, 'README.md'), `# ${name}

Koa.js API

## Getting Started

npm run dev
`)
}
