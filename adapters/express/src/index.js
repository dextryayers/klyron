import express from 'express'

export function createAdapter() {
  return {
    name: 'express',
    detect(dir) {
      try {
        const pkg = readPackageJson(dir)
        return pkg?.dependencies?.express || pkg?.devDependencies?.express
      } catch { return false }
    },
    supportedVersions: ['4.18', '5.0', '5.1'],
    defaultVersion: '5.1',
    kind: 'Backend',

    async dev(dir, port) {
      const { spawn } = await import('child_process')
      const hasTS = existsSync(join(dir, 'tsconfig.json'))
      const cmd = hasTS ? 'npx' : 'npx'
      const args = hasTS ? ['tsx', 'watch', 'src/index.ts'] : ['node', '--watch', 'src/index.js']
      const proc = spawn(cmd, args, { cwd: dir, env: { ...process.env, PORT: String(port || 3000) }, stdio: 'inherit' })
      return new Promise((resolve, reject) => {
        proc.on('close', (code) => code === 0 ? resolve() : reject(new Error(`Dev server exited with code ${code}`)))
      })
    },

    async build(dir) {
      const { execFile } = await import('child_process')
      const hasTS = existsSync(join(dir, 'tsconfig.json'))
      if (hasTS) {
        await execFile('npx', ['tsc'], { cwd: dir })
      }
    },

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
      return scaffoldExpress(name, options)
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

import { readFileSync, statSync, writeFileSync, mkdirSync } from 'fs'
import { join } from 'path'

async function scaffoldExpress(name, options) {
  const projectDir = join(options.dir || process.cwd(), name)
  mkdirSync(join(projectDir, 'src', 'routes'), { recursive: true })
  mkdirSync(join(projectDir, 'src', 'middleware'), { recursive: true })

  writeFileSync(join(projectDir, 'package.json'), JSON.stringify({
    name, version: '1.0.0', private: true, type: 'module',
    scripts: {
      dev: 'node --watch src/index.js',
      start: 'node src/index.js',
      test: 'jest',
      lint: 'eslint .',
      format: 'prettier --write .'
    },
    dependencies: { express: '^5.1.0', cors: '^2.8.5', morgan: '^1.10.0' },
    devDependencies: { jest: '^30.0.0', eslint: '^9.20.0', prettier: '^3.5.0' }
  }, null, 2))

  writeFileSync(join(projectDir, 'src', 'index.js'), `import express from 'express'
import cors from 'cors'
import morgan from 'morgan'
import routes from './routes/index.js'

const app = express()
const port = process.env.PORT || 3000

app.use(cors())
app.use(morgan('dev'))
app.use(express.json())
app.use('/', routes)

app.listen(port, () => console.log(\`${name} running on http://localhost:\${port}\`))
`)

  writeFileSync(join(projectDir, 'src', 'routes', 'index.js'), `import { Router } from 'express'

const router = Router()

router.get('/', (req, res) => {
  res.json({ message: 'Hello World' })
})

router.get('/health', (req, res) => {
  res.json({ status: 'ok', timestamp: new Date().toISOString() })
})

export default router
`)

  writeFileSync(join(projectDir, 'src', 'middleware', 'error.js'), `export function errorHandler(err, req, res, next) {
  console.error(err.stack)
  res.status(err.status || 500).json({
    error: err.message || 'Internal Server Error'
  })
}
`)

  writeFileSync(join(projectDir, '.gitignore'), 'node_modules\n.DS_Store\n')
  writeFileSync(join(projectDir, '.prettierrc'), JSON.stringify({ semi: true, singleQuote: true, tabWidth: 2, trailingComma: 'es5', printWidth: 100 }))
  writeFileSync(join(projectDir, 'eslint.config.js'), `import js from '@eslint/js'
export default [js.configs.recommended, { ignores: ['node_modules'] }]
`)
  writeFileSync(join(projectDir, 'README.md'), `# ${name}

Express.js API

## Getting Started

npm run dev
`)
}
