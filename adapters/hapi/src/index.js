import { readFileSync, statSync, writeFileSync, mkdirSync } from 'fs'
import { join } from 'path'

export function createAdapter() {
  return {
    name: 'hapi',
    detect(dir) {
      try {
        const pkg = readPackageJson(dir)
        return pkg?.dependencies?.['@hapi/hapi'] || pkg?.devDependencies?.['@hapi/hapi']
      } catch { return false }
    },
    supportedVersions: ['21.3'],
    defaultVersion: '21.3',
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
      return scaffoldHapi(name, options)
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

async function scaffoldHapi(name, options) {
  const projectDir = join(options.dir || process.cwd(), name)
  mkdirSync(join(projectDir, 'src', 'plugins'), { recursive: true })
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
    dependencies: { '@hapi/hapi': '^21.3.0' },
    devDependencies: { jest: '^30.0.0', eslint: '^9.20.0', prettier: '^3.5.0', nodemon: '^3.1.0' }
  }, null, 2))

  writeFileSync(join(projectDir, 'src', 'index.js'), `import Hapi from '@hapi/hapi'
import routes from './routes/index.js'

const init = async () => {
  const server = Hapi.server({
    port: process.env.PORT || 3000,
    host: '0.0.0.0'
  })

  await server.register(routes)

  await server.start()
  console.log('Server running on %s', server.info.uri)
}

process.on('unhandledRejection', (err) => {
  console.log(err)
  process.exit(1)
})

init()
`)

  writeFileSync(join(projectDir, 'src', 'routes', 'index.js'), `const routes = {
  name: 'routes',
  version: '1.0.0',
  register: async function (server, options) {
    server.route({
      method: 'GET',
      path: '/',
      handler: (request, h) => {
        return { message: 'Hello World' }
      }
    })

    server.route({
      method: 'GET',
      path: '/health',
      handler: (request, h) => {
        return { status: 'ok', timestamp: new Date().toISOString() }
      }
    })
  }
}

export default routes
`)

  writeFileSync(join(projectDir, 'src', 'plugins', 'logger.js'), `const logger = {
  name: 'logger',
  version: '1.0.0',
  register: async function (server, options) {
    server.events.on('response', (request) => {
      console.log(\`\${request.info.remoteAddress}: \${request.method.toUpperCase()} \${request.path} -> \${request.response.statusCode}\`)
    })
  }
}

export default logger
`)

  writeFileSync(join(projectDir, '.gitignore'), 'node_modules\n.DS_Store\n')
  writeFileSync(join(projectDir, '.prettierrc'), JSON.stringify({ semi: true, singleQuote: true, tabWidth: 2, trailingComma: 'es5', printWidth: 100 }))
  writeFileSync(join(projectDir, 'README.md'), `# ${name}

Hapi.js API

## Getting Started

npm run dev
`)
}
