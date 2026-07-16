import { readFileSync, statSync, writeFileSync, mkdirSync } from 'fs'
import { join } from 'path'

export function createAdapter() {
  return {
    name: 'adonis',
    detect(dir) {
      try {
        const pkg = readPackageJson(dir)
        const hasDep = pkg?.dependencies?.['@adonisjs/core'] || pkg?.devDependencies?.['@adonisjs/core']
        if (hasDep) return true
      } catch { /* ignore */ }
      return existsSync(join(dir, '.adonisrc.json')) || existsSync(join(dir, 'ace'))
    },
    supportedVersions: ['6.0'],
    defaultVersion: '6.0',
    kind: 'Backend',

    async dev(dir, port) {
      const { spawn } = await import('child_process')
      const proc = spawn('node', ['ace', 'serve', '--watch'], { cwd: dir, env: { ...process.env, PORT: String(port || 3333) }, stdio: 'inherit' })
      return new Promise((resolve, reject) => {
        proc.on('close', (code) => code === 0 ? resolve() : reject(new Error(`Dev server exited with code ${code}`)))
      })
    },

    async build(dir) {
      const { execFile } = await import('child_process')
      await execFile('node', ['ace', 'build'], { cwd: dir })
    },

    async test(dir) {
      const { execFile } = await import('child_process')
      await execFile('node', ['ace', 'test'], { cwd: dir })
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
      return scaffoldAdonis(name, options)
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

async function scaffoldAdonis(name, options) {
  const projectDir = join(options.dir || process.cwd(), name)
  mkdirSync(join(projectDir, 'app', 'Controllers', 'Http'), { recursive: true })
  mkdirSync(join(projectDir, 'config'), { recursive: true })
  mkdirSync(join(projectDir, 'database', 'migrations'), { recursive: true })
  mkdirSync(join(projectDir, 'start'), { recursive: true })

  writeFileSync(join(projectDir, 'package.json'), JSON.stringify({
    name, version: '1.0.0', private: true, type: 'module',
    scripts: {
      dev: 'node ace serve --watch',
      build: 'node ace build',
      test: 'node ace test',
      lint: 'eslint .',
      format: 'prettier --write .'
    },
    dependencies: { '@adonisjs/core': '^6.0.0', '@adonisjs/lucid': '^21.0.0' },
    devDependencies: { eslint: '^9.20.0', prettier: '^3.5.0' }
  }, null, 2))

  writeFileSync(join(projectDir, '.env'), `PORT=3333
HOST=0.0.0.0
NODE_ENV=development
`)

  writeFileSync(join(projectDir, 'start', 'routes.js'), `import router from '@adonisjs/core/services/router'

router.get('/', async () => {
  return { hello: 'world' }
})
`)

  writeFileSync(join(projectDir, 'config', 'app.js'), `import env from '#start/env'
import { defineConfig } from '@adonisjs/core'

export default defineConfig({
  appKey: env.get('APP_KEY'),
  http: {
    host: env.get('HOST', '0.0.0.0'),
    port: env.get('PORT', 3333)
  }
})
`)

  writeFileSync(join(projectDir, 'app', 'Controllers', 'Http', 'HelloController.js'), `export default class HelloController {
  async index({ request }) {
    return { greeting: 'Hello from AdonisJS' }
  }
}
`)

  writeFileSync(join(projectDir, '.adonisrc.json'), JSON.stringify({
    typescript: false,
    directories: { controllers: 'app/Controllers', config: 'config', database: 'database', migrations: 'database/migrations' }
  }, null, 2))

  writeFileSync(join(projectDir, '.gitignore'), 'node_modules\n.env\nbuild\n.DS_Store\ntmp\n')
  writeFileSync(join(projectDir, '.prettierrc'), JSON.stringify({ semi: true, singleQuote: true, tabWidth: 2, trailingComma: 'es5', printWidth: 100 }))
  writeFileSync(join(projectDir, 'README.md'), `# ${name}

AdonisJS API

## Getting Started

npm run dev
`)
}
