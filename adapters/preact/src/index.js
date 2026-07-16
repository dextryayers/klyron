import { readFileSync, statSync, writeFileSync, mkdirSync } from 'fs'
import { join } from 'path'

export function createAdapter() {
  return {
    name: 'preact',
    detect(dir) {
      try {
        const pkg = readPackageJson(dir)
        return pkg?.dependencies?.preact || pkg?.devDependencies?.preact
      } catch { return false }
    },
    supportedVersions: ['10.0'],
    defaultVersion: '10.0',
    kind: 'Frontend',

    async dev(dir, port) {
      const { spawn } = await import('child_process')
      const proc = spawn('npx', ['vite'], {
        cwd: dir,
        env: { ...process.env, PORT: String(port || 5173) },
        stdio: 'inherit'
      })
      return new Promise((resolve, reject) => {
        proc.on('close', (code) => code === 0 ? resolve() : reject(new Error(`Dev server exited with code ${code}`)))
      })
    },

    async build(dir) {
      const { execFile } = await import('child_process')
      await execFile('npx', ['vite', 'build'], { cwd: dir })
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
      return scaffoldPreact(name, options)
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

async function scaffoldPreact(name, options) {
  const projectDir = join(options.dir || process.cwd(), name)
  mkdirSync(join(projectDir, 'src', 'components'), { recursive: true })
  mkdirSync(join(projectDir, 'public'), { recursive: true })

  writeFileSync(join(projectDir, 'package.json'), JSON.stringify({
    name, version: '1.0.0', private: true, type: 'module',
    scripts: { dev: 'vite', build: 'vite build', preview: 'vite preview', test: 'vitest run', lint: 'eslint .', format: 'prettier --write .' },
    dependencies: { preact: '^10.26.0' },
    devDependencies: { '@preact/preset-vite': '^2.10.0', vite: '^6.1.0', typescript: '^5.7.0', vitest: '^3.0.0', eslint: '^9.20.0', '@eslint/js': '^9.20.0', prettier: '^3.5.0' }
  }, null, 2))

  writeFileSync(join(projectDir, 'tsconfig.json'), JSON.stringify({
    compilerOptions: {
      target: 'ES2022', module: 'ESNext', moduleResolution: 'bundler', strict: true, jsx: 'react-jsx',
      jsxImportSource: 'preact', skipLibCheck: true, noEmit: true, isolatedModules: true,
      paths: { react: ['./node_modules/preact/compat'], 'react-dom': ['./node_modules/preact/compat'] }
    },
    include: ['src']
  }, null, 2))

  writeFileSync(join(projectDir, 'vite.config.ts'), `import { defineConfig } from 'vite'
import preact from '@preact/preset-vite'

export default defineConfig({
  plugins: [preact()],
  server: { port: 5173, host: true },
})
`)

  writeFileSync(join(projectDir, 'index.html'), `<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>${name}</title>
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
`)

  writeFileSync(join(projectDir, 'src', 'main.tsx'), `import { render } from 'preact'
import App from './app'

render(<App />, document.getElementById('app')!)
`)

  writeFileSync(join(projectDir, 'src', 'app.tsx'), `import { useState } from 'preact/hooks'

export function App() {
  const [count, setCount] = useState(0)

  return (
    <div>
      <h1>Welcome to ${name}</h1>
      <p>Built with Preact</p>
      <button onClick={() => setCount(c => c + 1)}>Count: {count}</button>
    </div>
  )
}
`)

  writeFileSync(join(projectDir, 'src', 'index.css'), `* { margin: 0; padding: 0; box-sizing: border-box; }
:root { font-family: Inter, system-ui, Avenir, Helvetica, Arial, sans-serif; line-height: 1.5; color: #213547; background-color: #fff; }
body { min-height: 100vh; }
h1 { font-size: 3.2em; line-height: 1.1; }
button { padding: 0.5em 1em; font-size: 1em; cursor: pointer; }
`)

  writeFileSync(join(projectDir, 'src', 'vite-env.d.ts'), `/// <reference types="vite/client" />`)
  writeFileSync(join(projectDir, '.gitignore'), 'node_modules\ndist\n.DS_Store\n*.tsbuildinfo\n')
  writeFileSync(join(projectDir, '.prettierrc'), JSON.stringify({ semi: true, singleQuote: true, tabWidth: 2, trailingComma: 'es5', printWidth: 100 }))
  writeFileSync(join(projectDir, 'eslint.config.js'), `import js from '@eslint/js'
export default [js.configs.recommended, { ignores: ['node_modules', 'dist'] }]
`)
  writeFileSync(join(projectDir, 'README.md'), `# ${name}\n\nPreact App\n\nnpm run dev\n`)
}
