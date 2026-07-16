import { readFileSync, statSync, writeFileSync, mkdirSync } from 'fs'
import { join } from 'path'

export function createAdapter() {
  return {
    name: 'trpc',
    detect(dir) {
      try {
        const pkg = readPackageJson(dir)
        return pkg?.dependencies?.['@trpc/server'] || pkg?.devDependencies?.['@trpc/server']
      } catch { return false }
    },
    supportedVersions: ['10.45', '11.0'],
    defaultVersion: '11.0',
    kind: 'ApiFramework',

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

    async format(dir, write) {
      const { execFile } = await import('child_process')
      await execFile('npx', write ? ['prettier', '--write', '.'] : ['prettier', '--check', '.'], { cwd: dir })
    },

    scaffold(name, options) {
      return scaffoldTrpc(name, options)
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

async function scaffoldTrpc(name, options) {
  const projectDir = join(options.dir || process.cwd(), name)
  mkdirSync(join(projectDir, 'src', 'router'), { recursive: true })
  mkdirSync(join(projectDir, 'src', 'context'), { recursive: true })

  writeFileSync(join(projectDir, 'package.json'), JSON.stringify({
    name, version: '1.0.0', private: true, type: 'module',
    scripts: {
      dev: 'tsx watch src/index.ts',
      build: 'tsc',
      test: 'vitest run',
      lint: 'eslint .',
      format: 'prettier --write .'
    },
    dependencies: { '@trpc/server': '^11.0.0', '@trpc/client': '^11.0.0', zod: '^3.23.0' },
    devDependencies: { typescript: '^5.6.0', tsx: '^4.19.0', vitest: '^2.1.0', eslint: '^9.20.0', prettier: '^3.5.0' }
  }, null, 2))

  writeFileSync(join(projectDir, 'tsconfig.json'), JSON.stringify({
    compilerOptions: {
      target: 'ES2022', module: 'ES2022', moduleResolution: 'bundler',
      strict: true, esModuleInterop: true, outDir: 'dist',
      rootDir: 'src', declaration: true
    },
    include: ['src']
  }, null, 2))

  writeFileSync(join(projectDir, 'src', 'index.ts'), `import { createHTTPServer } from '@trpc/server/adapters/standalone'
import { appRouter } from './router/index.js'
import { createContext } from './context/index.js'

const port = process.env.PORT || 3000

const server = createHTTPServer({
  router: appRouter,
  createContext
})

server.listen(port, () => {
  console.log(\`tRPC server running on http://localhost:\${port}\`)
})
`)

  writeFileSync(join(projectDir, 'src', 'router', 'index.ts'), `import { initTRPC } from '@trpc/server'
import { z } from 'zod'
import type { Context } from '../context/index.js'

const t = initTRPC.context<Context>().create()

export const appRouter = t.router({
  greet: t.procedure
    .input(z.object({ name: z.string() }))
    .query(({ input }) => {
      return \`Hello, \${input.name}!\`
    }),
  health: t.procedure
    .query(() => {
      return { status: 'ok', timestamp: new Date().toISOString() }
    })
})

export type AppRouter = typeof appRouter
`)

  writeFileSync(join(projectDir, 'src', 'context', 'index.ts'), `import { inferAsyncReturnType } from '@trpc/server'
import { CreateHTTPContextOptions } from '@trpc/server/adapters/standalone'

export async function createContext(opts: CreateHTTPContextOptions) {
  return {
    req: opts.req,
    res: opts.res
  }
}

export type Context = inferAsyncReturnType<typeof createContext>
`)

  writeFileSync(join(projectDir, '.gitignore'), 'node_modules\n.DS_Store\ndist\n')
  writeFileSync(join(projectDir, '.prettierrc'), JSON.stringify({ semi: true, singleQuote: true, tabWidth: 2, trailingComma: 'es5', printWidth: 100 }))
  writeFileSync(join(projectDir, 'README.md'), `# ${name}

tRPC API

## Getting Started

npm run dev
`)
}
