import { readFileSync, statSync, writeFileSync, mkdirSync } from 'fs'
import { join } from 'path'

export function createAdapter() {
  return {
    name: 'svelte',
    detect(dir) {
      try {
        const hasVite = existsSync(join(dir, 'vite.config.ts')) || existsSync(join(dir, 'vite.config.js'))
        const pkg = readPackageJson(dir)
        const hasSvelte = pkg?.dependencies?.svelte || pkg?.devDependencies?.svelte
        const hasKit = pkg?.dependencies?.['@sveltejs/kit'] || pkg?.devDependencies?.['@sveltejs/kit']
        return hasVite && hasSvelte && !hasKit
      } catch { return false }
    },
    supportedVersions: ['4.2', '5.0'],
    defaultVersion: '5.0',
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
      return scaffoldSvelte(name, options)
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

async function scaffoldSvelte(name, options) {
  const projectDir = join(options.dir || process.cwd(), name)
  mkdirSync(join(projectDir, 'src', 'lib'), { recursive: true })
  mkdirSync(join(projectDir, 'src', 'assets'), { recursive: true })
  mkdirSync(join(projectDir, 'public'), { recursive: true })

  writeFileSync(join(projectDir, 'package.json'), JSON.stringify({
    name, version: '1.0.0', private: true, type: 'module',
    scripts: { dev: 'vite', build: 'vite build', preview: 'vite preview', check: 'svelte-check', test: 'vitest run', lint: 'eslint .', format: 'prettier --write .' },
    dependencies: { svelte: '^5.0.0' },
    devDependencies: {
      '@sveltejs/vite-plugin-svelte': '^5.0.0', vite: '^6.1.0', typescript: '^5.7.0',
      'svelte-check': '^4.1.0', vitest: '^3.0.0', jsdom: '^26.0.0', eslint: '^9.20.0',
      '@eslint/js': '^9.20.0', 'typescript-eslint': '^8.24.0',
      'eslint-plugin-svelte': '^3.0.0', prettier: '^3.5.0', 'prettier-plugin-svelte': '^3.3.0'
    }
  }, null, 2))

  writeFileSync(join(projectDir, 'tsconfig.json'), JSON.stringify({
    compilerOptions: {
      target: 'ES2022', module: 'ESNext', moduleResolution: 'bundler',
      strict: true, skipLibCheck: true, jsx: 'preserve', noEmit: true,
      isolatedModules: true, resolveJsonModule: true, allowImportingTsExtensions: true
    },
    include: ['src/**/*.ts', 'src/**/*.svelte'],
    exclude: ['node_modules']
  }, null, 2))

  writeFileSync(join(projectDir, 'vite.config.ts'), `import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'

export default defineConfig({
  plugins: [svelte()],
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
    <script type="module" src="/src/main.ts"></script>
  </body>
</html>
`)

  writeFileSync(join(projectDir, 'src', 'main.ts'), `import App from './App.svelte'
import { mount } from 'svelte'

const app = mount(App, { target: document.getElementById('app')! })

export default app
`)

  writeFileSync(join(projectDir, 'src', 'App.svelte'), `<script lang="ts">
  import Home from './lib/Home.svelte'
</script>

<main>
  <Home />
</main>

<style>
  main { min-height: 100vh; display: flex; flex-direction: column; align-items: center; justify-content: center; }
</style>
`)

  writeFileSync(join(projectDir, 'src', 'lib', 'Home.svelte'), `<script lang="ts">
  let count = \$state(0)
</script>

<h1>Welcome to {name}</h1>
<button onclick={() => count++}>count is {count}</button>

<style>
  h1 { font-size: 3.2em; line-height: 1.1; }
  button { border-radius: 8px; border: 1px solid transparent; padding: 0.6em 1.2em; font-size: 1em; font-weight: 500; cursor: pointer; }
</style>
`.replace('{name}', name))

  writeFileSync(join(projectDir, 'src', 'app.css'), `* { margin: 0; padding: 0; box-sizing: border-box; }
:root { font-family: Inter, system-ui, Avenir, Helvetica, Arial, sans-serif; line-height: 1.5; font-weight: 400; color: #213547; background-color: #fff; }
body { min-height: 100vh; }
`)

  writeFileSync(join(projectDir, 'svelte.config.js'), `import { vitePreprocess } from '@sveltejs/vite-plugin-svelte'

export default {
  preprocess: vitePreprocess(),
}
`)

  writeFileSync(join(projectDir, '.gitignore'), 'node_modules\ndist\n.DS_Store\n')
  writeFileSync(join(projectDir, '.prettierrc'), JSON.stringify({ semi: false, singleQuote: true, tabWidth: 2, trailingComma: 'none', printWidth: 100 }))
  writeFileSync(join(projectDir, 'README.md'), `# ${name}\n\nSvelte + Vite\n\nnpm run dev\n`)
}
