import { readFileSync, statSync, writeFileSync, mkdirSync } from 'fs'
import { join } from 'path'

export function createAdapter() {
  return {
    name: 'react',
    detect(dir) {
      try {
        const hasVite = existsSync(join(dir, 'vite.config.ts')) || existsSync(join(dir, 'vite.config.js')) || existsSync(join(dir, 'vite.config.mjs'))
        const pkg = readPackageJson(dir)
        return hasVite && (pkg?.dependencies?.react || pkg?.dependencies?.['react-dom'])
      } catch { return false }
    },
    supportedVersions: ['18.0', '19.0', '19.1'],
    defaultVersion: '19.1',
    kind: 'Frontend',

    async dev(dir, port) {
      const { spawn } = await import('child_process')
      const proc = spawn('npx', ['vite'], { cwd: dir, env: { ...process.env, PORT: String(port || 5173) }, stdio: 'inherit' })
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
      return scaffoldReact(name, options)
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

async function scaffoldReact(name, options) {
  const projectDir = join(options.dir || process.cwd(), name)
  mkdirSync(join(projectDir, 'src', 'pages'), { recursive: true })
  mkdirSync(join(projectDir, 'src', 'assets'), { recursive: true })
  mkdirSync(join(projectDir, 'public'), { recursive: true })

  writeFileSync(join(projectDir, 'package.json'), JSON.stringify({
    name, private: true, version: '1.0.0', type: 'module',
    scripts: { dev: 'vite', build: 'vite build', preview: 'vite preview', test: 'vitest run', lint: 'eslint .', format: 'prettier --write .' },
    dependencies: { react: '^19.1.0', 'react-dom': '^19.1.0', 'react-router-dom': '^7.1.0' },
    devDependencies: {
      '@vitejs/plugin-react': '^4.4.0', vite: '^6.1.0', typescript: '^5.7.0',
      '@types/react': '^19.1.0', '@types/react-dom': '^19.1.0', vitest: '^3.0.0',
      '@testing-library/react': '^16.2.0', jsdom: '^26.0.0', eslint: '^9.20.0',
      '@eslint/js': '^9.20.0', 'typescript-eslint': '^8.24.0',
      'eslint-plugin-react-hooks': '^5.2.0', 'eslint-plugin-react-refresh': '^0.4.19',
      prettier: '^3.5.0', 'prettier-plugin-tailwindcss': '^0.6.11'
    }
  }, null, 2))

  writeFileSync(join(projectDir, 'tsconfig.json'), JSON.stringify({
    compilerOptions: {
      target: 'ES2022', useDefineForClassFields: true,
      lib: ['ES2023', 'DOM', 'DOM.Iterable'], module: 'ESNext', skipLibCheck: true,
      moduleResolution: 'bundler', allowImportingTsExtensions: true,
      isolatedModules: true, moduleDetection: 'force', noEmit: true,
      jsx: 'react-jsx', strict: true, noUnusedLocals: true, noUnusedParameters: true,
      noFallthroughCasesInSwitch: true, forceConsistentCasingInFileNames: true
    },
    include: ['src']
  }, null, 2))

  writeFileSync(join(projectDir, 'vite.config.ts'), `import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
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
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
`)

  writeFileSync(join(projectDir, 'src', 'main.tsx'), `import React from 'react'
import ReactDOM from 'react-dom/client'
import { BrowserRouter } from 'react-router-dom'
import App from './App'
import './index.css'

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <BrowserRouter><App /></BrowserRouter>
  </React.StrictMode>,
)
`)

  writeFileSync(join(projectDir, 'src', 'App.tsx'), `import { Routes, Route } from 'react-router-dom'
import Home from './pages/Home'

function App() {
  return (
    <Routes>
      <Route path="/" element={<Home />} />
    </Routes>
  )
}
export default App
`)

  writeFileSync(join(projectDir, 'src', 'pages', 'Home.tsx'), `function Home() {
  return (
    <div>
      <h1>Welcome to ${name}</h1>
    </div>
  )
}
export default Home
`)

  writeFileSync(join(projectDir, 'src', 'index.css'), `* { margin: 0; padding: 0; box-sizing: border-box; }
:root { font-family: Inter, system-ui, Avenir, Helvetica, Arial, sans-serif; line-height: 1.5; font-weight: 400; color: #213547; background-color: #fff; }
body { min-height: 100vh; }
a { font-weight: 500; color: #646cff; text-decoration: inherit; }
a:hover { color: #535bf2; }
h1 { font-size: 3.2em; line-height: 1.1; }
`)

  writeFileSync(join(projectDir, 'src', 'vite-env.d.ts'), `/// <reference types="vite/client" />`)
  writeFileSync(join(projectDir, '.gitignore'), 'node_modules\ndist\n.DS_Store\n*.local\n')
  writeFileSync(join(projectDir, '.prettierrc'), JSON.stringify({ semi: true, singleQuote: true, tabWidth: 2, trailingComma: 'es5', printWidth: 100 }))
  writeFileSync(join(projectDir, 'eslint.config.js'), `import js from '@eslint/js'
import tseslint from 'typescript-eslint'
import reactHooks from 'eslint-plugin-react-hooks'
import reactRefresh from 'eslint-plugin-react-refresh'

export default tseslint.config(
  { ignores: ['dist'] },
  {
    extends: [js.configs.recommended, ...tseslint.configs.recommended],
    files: ['**/*.{ts,tsx}'],
    plugins: { 'react-hooks': reactHooks, 'react-refresh': reactRefresh },
    rules: { ...reactHooks.configs.recommended.rules, 'react-refresh/only-export-components': ['warn', { allowConstantExport: true }] }
  }
)
`)
  writeFileSync(join(projectDir, 'README.md'), `# ${name}\n\nReact + Vite + TypeScript\n\nnpm run dev\n`)
}
