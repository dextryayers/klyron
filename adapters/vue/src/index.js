import { readFileSync, statSync, writeFileSync, mkdirSync } from 'fs'
import { join } from 'path'

export function createAdapter() {
  return {
    name: 'vue',
    detect(dir) {
      try {
        const hasVite = existsSync(join(dir, 'vite.config.ts')) || existsSync(join(dir, 'vite.config.js'))
        const pkg = readPackageJson(dir)
        return hasVite && (pkg?.dependencies?.vue || pkg?.dependencies?.['vue-router'] || pkg?.dependencies?.pinia)
      } catch { return false }
    },
    supportedVersions: ['3.4', '3.5', '3.6'],
    defaultVersion: '3.6',
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
      return scaffoldVue(name, options)
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

async function scaffoldVue(name, options) {
  const projectDir = join(options.dir || process.cwd(), name)
  mkdirSync(join(projectDir, 'src', 'views'), { recursive: true })
  mkdirSync(join(projectDir, 'src', 'components'), { recursive: true })
  mkdirSync(join(projectDir, 'src', 'assets'), { recursive: true })
  mkdirSync(join(projectDir, 'public'), { recursive: true })

  writeFileSync(join(projectDir, 'package.json'), JSON.stringify({
    name, private: true, version: '1.0.0', type: 'module',
    scripts: { dev: 'vite', build: 'vite build', preview: 'vite preview', test: 'vitest run', lint: 'eslint .', format: 'prettier --write .' },
    dependencies: { vue: '^3.6.0', 'vue-router': '^4.5.0', pinia: '^3.0.0' },
    devDependencies: {
      '@vitejs/plugin-vue': '^5.2.0', vite: '^6.1.0', typescript: '^5.7.0',
      vue: '^3.6.0', vitest: '^3.0.0', jsdom: '^26.0.0', eslint: '^9.20.0',
      '@eslint/js': '^9.20.0', 'typescript-eslint': '^8.24.0',
      'eslint-plugin-vue': '^9.32.0', prettier: '^3.5.0'
    }
  }, null, 2))

  writeFileSync(join(projectDir, 'tsconfig.json'), JSON.stringify({
    compilerOptions: {
      target: 'ES2022', module: 'ESNext', moduleResolution: 'bundler',
      strict: true, jsx: 'preserve', skipLibCheck: true, noEmit: true,
      isolatedModules: true, resolveJsonModule: true, allowImportingTsExtensions: true
    },
    include: ['src/**/*.ts', 'src/**/*.vue'],
    exclude: ['node_modules']
  }, null, 2))

  writeFileSync(join(projectDir, 'vite.config.ts'), `import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

export default defineConfig({
  plugins: [vue()],
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

  writeFileSync(join(projectDir, 'src', 'main.ts'), `import { createApp } from 'vue'
import { createPinia } from 'pinia'
import router from './router'
import App from './App.vue'
import './assets/main.css'

const app = createApp(App)
app.use(createPinia())
app.use(router)
app.mount('#app')
`)

  writeFileSync(join(projectDir, 'src', 'App.vue'), `<script setup lang="ts">
import { RouterView } from 'vue-router'
</script>

<template>
  <RouterView />
</template>
`)

  writeFileSync(join(projectDir, 'src', 'router', 'index.ts'), `import { createRouter, createWebHistory } from 'vue-router'
import HomeView from '../views/HomeView.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    { path: '/', name: 'home', component: HomeView },
  ]
})

export default router
`)

  writeFileSync(join(projectDir, 'src', 'views', 'HomeView.vue'), `<script setup lang="ts">
</script>

<template>
  <main>
    <h1>Welcome to ${name}</h1>
  </main>
</template>

<style scoped>
h1 { font-size: 3.2em; line-height: 1.1; }
</style>
`)

  writeFileSync(join(projectDir, 'src', 'assets', 'main.css'), `* { margin: 0; padding: 0; box-sizing: border-box; }
:root { font-family: Inter, system-ui, Avenir, Helvetica, Arial, sans-serif; line-height: 1.5; font-weight: 400; color: #213547; background-color: #fff; }
body { min-height: 100vh; }
`)

  writeFileSync(join(projectDir, 'src', 'env.d.ts'), `/// <reference types="vite/client" />
declare module '*.vue' {
  import type { DefineComponent } from 'vue'
  const component: DefineComponent<{}, {}, any>
  export default component
}
`)
  writeFileSync(join(projectDir, '.gitignore'), 'node_modules\ndist\n.DS_Store\n*.local\n')
  writeFileSync(join(projectDir, '.prettierrc'), JSON.stringify({ semi: false, singleQuote: true, tabWidth: 2, trailingComma: 'none', printWidth: 100 }))
  writeFileSync(join(projectDir, 'README.md'), `# ${name}\n\nVue 3 + Vite + TypeScript\n\nnpm run dev\n`)
}
