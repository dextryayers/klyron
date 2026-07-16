import { readFileSync, statSync, writeFileSync, mkdirSync } from 'fs'
import { join } from 'path'

export function createAdapter() {
  return {
    name: 'solidstart',
    detect(dir) {
      try {
        const pkg = readPackageJson(dir)
        return pkg?.dependencies?.['solid-js'] || pkg?.devDependencies?.['solid-js']
      } catch { return false }
    },
    supportedVersions: ['1.0'],
    defaultVersion: '1.0',
    kind: 'Fullstack',

    async dev(dir, port) {
      const { spawn } = await import('child_process')
      const proc = spawn('npx', ['vinxi', 'dev'], {
        cwd: dir,
        env: { ...process.env, PORT: String(port || 3000) },
        stdio: 'inherit'
      })
      return new Promise((resolve, reject) => {
        proc.on('close', (code) => code === 0 ? resolve() : reject(new Error(`Dev server exited with code ${code}`)))
      })
    },

    async build(dir) {
      const { execFile } = await import('child_process')
      await execFile('npx', ['vinxi', 'build'], { cwd: dir })
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
      return scaffoldSolidStart(name, options)
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

async function scaffoldSolidStart(name, options) {
  const projectDir = join(options.dir || process.cwd(), name)
  mkdirSync(join(projectDir, 'src', 'routes'), { recursive: true })
  mkdirSync(join(projectDir, 'src', 'components'), { recursive: true })
  mkdirSync(join(projectDir, 'public'), { recursive: true })

  writeFileSync(join(projectDir, 'package.json'), JSON.stringify({
    name, version: '1.0.0', private: true, type: 'module',
    scripts: { dev: 'vinxi dev', build: 'vinxi build', start: 'vinxi start', test: 'vitest run', lint: 'eslint .', format: 'prettier --write .' },
    dependencies: { 'solid-js': '^1.9.0', '@solidjs/start': '^1.0.0', '@solidjs/router': '^0.15.0' },
    devDependencies: { vinxi: '^0.5.0', vite: '^6.1.0', typescript: '^5.7.0', vitest: '^3.0.0', eslint: '^9.20.0', '@eslint/js': '^9.20.0', prettier: '^3.5.0' }
  }, null, 2))

  writeFileSync(join(projectDir, 'app.config.ts'), `import { defineConfig } from '@solidjs/start/config'

export default defineConfig({})
`)

  writeFileSync(join(projectDir, 'tsconfig.json'), JSON.stringify({
    compilerOptions: {
      target: 'ESNext', module: 'ESNext', moduleResolution: 'bundler', strict: true, jsx: 'react-jsx',
      jsxImportSource: 'solid-js', types: ['vinxi/types/client'], skipLibCheck: true, noEmit: true,
      isolatedModules: true
    },
    include: ['src']
  }, null, 2))

  writeFileSync(join(projectDir, 'src', 'routes', 'index.tsx'), `import { Title } from '@solidjs/meta'

export default function Home() {
  return (
    <main>
      <Title>${name}</Title>
      <h1>Welcome to ${name}</h1>
      <p>Built with SolidStart</p>
    </main>
  )
}
`)

  writeFileSync(join(projectDir, 'src', 'entry-client.tsx'), `import { mount, StartClient } from '@solidjs/start/client'

mount(() => <StartClient />, document.getElementById('app'))
`)

  writeFileSync(join(projectDir, 'src', 'entry-server.tsx'), `import { createHandler, StartServer } from '@solidjs/start/server'

export default createHandler(() => <StartServer />)
`)

  writeFileSync(join(projectDir, 'src', 'app.tsx'), `import { Router } from '@solidjs/router'
import { FileRoutes } from '@solidjs/start/router'
import { Suspense } from 'solid-js'

export default function App() {
  return (
    <Router root={(props) => <Suspense>{props.children}</Suspense>}>
      <FileRoutes />
    </Router>
  )
}
`)

  writeFileSync(join(projectDir, '.gitignore'), 'node_modules\n.output\n.DS_Store\n*.tsbuildinfo\n')
  writeFileSync(join(projectDir, '.prettierrc'), JSON.stringify({ semi: true, singleQuote: true, tabWidth: 2, trailingComma: 'es5', printWidth: 100 }))
  writeFileSync(join(projectDir, 'eslint.config.js'), `import js from '@eslint/js'
export default [js.configs.recommended, { ignores: ['node_modules', '.output'] }]
`)
  writeFileSync(join(projectDir, 'README.md'), `# ${name}\n\nSolidStart App\n\nnpm run dev\n`)
}
