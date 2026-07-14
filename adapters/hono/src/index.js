import { readFileSync, statSync, writeFileSync, mkdirSync } from 'fs'
import { join } from 'path'

export function createAdapter() {
  return {
    name: 'hono',
    detect(dir) {
      try {
        const pkg = readPackageJson(dir)
        return pkg?.dependencies?.hono || pkg?.devDependencies?.hono
      } catch { return false }
    },
    supportedVersions: ['4.6', '4.7'],
    defaultVersion: '4.7',
    kind: 'Backend',

    async dev(dir, port) {
      const { spawn } = await import('child_process')
      const proc = spawn('npx', ['tsx', 'watch', 'src/index.ts'], { cwd: dir, env: { ...process.env, PORT: String(port || 3000) }, stdio: 'inherit' })
      return new Promise((resolve, reject) => {
        proc.on('close', (code) => code === 0 ? resolve() : reject(new Error(`Dev server exited with code ${code}`)))
      })
    },

    async build(dir) {
      const { execFile } = await import('child_process')
      await execFile('npx', ['tsc'], { cwd: dir })
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
      return scaffoldHono(name, options)
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

async function scaffoldHono(name, options) {
  const projectDir = join(options.dir || process.cwd(), name)
  mkdirSync(join(projectDir, 'src', 'routes'), { recursive: true })

  writeFileSync(join(projectDir, 'package.json'), JSON.stringify({
    name, version: '1.0.0', private: true, type: 'module',
    scripts: { dev: 'tsx watch src/index.ts', start: 'tsx src/index.ts', test: 'vitest run', lint: 'eslint .', format: 'prettier --write .' },
    dependencies: { hono: '^4.7.0', '@hono/zod-validator': '^0.4.0' },
    devDependencies: { typescript: '^5.7.0', tsx: '^4.19.0', vitest: '^3.0.0', eslint: '^9.20.0', prettier: '^3.5.0' }
  }, null, 2))

  writeFileSync(join(projectDir, 'tsconfig.json'), JSON.stringify({
    compilerOptions: { target: 'ES2022', module: 'ESNext', moduleResolution: 'bundler', strict: true, skipLibCheck: true, noEmit: true, isolatedModules: true },
    include: ['src']
  }, null, 2))

  writeFileSync(join(projectDir, 'src', 'index.ts'), `import { Hono } from 'hono'
import { logger } from 'hono/logger'
import { cors } from 'hono/cors'
import routes from './routes/index'

const app = new Hono()

app.use('*', logger())
app.use('*', cors())
app.route('/', routes)

export default { port: process.env.PORT || 3000, fetch: app.fetch }

console.log(\`${name} running on http://localhost:\${process.env.PORT || 3000}\`)
`)

  writeFileSync(join(projectDir, 'src', 'routes', 'index.ts'), `import { Hono } from 'hono'

const router = new Hono()

router.get('/', (c) => c.json({ message: 'Hello World' }))

router.get('/health', (c) => c.json({ status: 'ok', timestamp: new Date().toISOString() }))

export default router
`)

  writeFileSync(join(projectDir, '.gitignore'), 'node_modules\n.DS_Store\n')
  writeFileSync(join(projectDir, '.prettierrc'), JSON.stringify({ semi: true, singleQuote: true, tabWidth: 2, trailingComma: 'es5', printWidth: 100 }))
  writeFileSync(join(projectDir, 'eslint.config.js'), `import js from '@eslint/js'
import tseslint from 'typescript-eslint'
export default tseslint.config(js.configs.recommended, ...tseslint.configs.recommended, { ignores: ['node_modules'] })
`)
  writeFileSync(join(projectDir, 'README.md'), `# ${name}\n\nHono API\n\nnpm run dev\n`)
}
