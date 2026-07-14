import { readFileSync, statSync, writeFileSync, mkdirSync } from 'fs'
import { join } from 'path'

export function createAdapter() {
  return {
    name: 'nuxt',
    detect(dir) {
      return existsSync(join(dir, 'nuxt.config.ts')) || existsSync(join(dir, 'nuxt.config.js')) || existsSync(join(dir, 'nuxt.config.mjs'))
    },
    supportedVersions: ['3.13'],
    defaultVersion: '3.13',
    kind: 'Fullstack',

    async dev(dir, port) {
      const { spawn } = await import('child_process')
      const proc = spawn('npx', ['nuxt', 'dev'], { cwd: dir, env: { ...process.env, PORT: String(port || 3000) }, stdio: 'inherit' })
      return new Promise((resolve, reject) => {
        proc.on('close', (code) => code === 0 ? resolve() : reject(new Error(`Dev server exited with code ${code}`)))
      })
    },

    async build(dir) {
      const { execFile } = await import('child_process')
      await execFile('npx', ['nuxt', 'build'], { cwd: dir })
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
      return scaffoldNuxt(name, options)
    }
  }
}

function existsSync(p) {
  try { return statSync(p).isFile() } catch { return false }
}

async function scaffoldNuxt(name, options) {
  const projectDir = join(options.dir || process.cwd(), name)
  mkdirSync(join(projectDir, 'pages'), { recursive: true })
  mkdirSync(join(projectDir, 'components'), { recursive: true })
  mkdirSync(join(projectDir, 'layouts'), { recursive: true })
  mkdirSync(join(projectDir, 'public'), { recursive: true })

  writeFileSync(join(projectDir, 'package.json'), JSON.stringify({
    name, private: true, version: '1.0.0', type: 'module',
    scripts: { dev: 'nuxt dev', build: 'nuxt build', generate: 'nuxt generate', preview: 'nuxt preview', lint: 'eslint .', format: 'prettier --write .' },
    dependencies: { nuxt: '^3.13.0', vue: '^3.6.0' },
    devDependencies: { 'nuxt': '^3.13.0', 'eslint': '^9.20.0', 'prettier': '^3.5.0' }
  }, null, 2))

  writeFileSync(join(projectDir, 'nuxt.config.ts'), `export default defineNuxtConfig({
  devtools: { enabled: true },
  css: ['~/assets/css/main.css'],
  postcss: {
    plugins: { tailwindcss: {}, autoprefixer: {} }
  }
})
`)

  writeFileSync(join(projectDir, 'tsconfig.json'), JSON.stringify({
    extends: './.nuxt/tsconfig.json',
    compilerOptions: { strict: true }
  }, null, 2))

  writeFileSync(join(projectDir, 'app.vue'), `<template>
  <NuxtLayout>
    <NuxtPage />
  </NuxtLayout>
</template>
`)

  writeFileSync(join(projectDir, 'pages', 'index.vue'), `<template>
  <main>
    <h1>Welcome to ${name}</h1>
  </main>
</template>

<style scoped>
h1 { font-size: 3.2em; line-height: 1.1; }
</style>
`)

  writeFileSync(join(projectDir, 'layouts', 'default.vue'), `<template>
  <div>
    <header>
      <nav>
        <NuxtLink to="/">${name}</NuxtLink>
      </nav>
    </header>
    <slot />
  </div>
</template>
`)

  writeFileSync(join(projectDir, 'assets', 'css', 'main.css'), `* { margin: 0; padding: 0; box-sizing: border-box; }
body { font-family: Inter, system-ui, Avenir, Helvetica, Arial, sans-serif; line-height: 1.5; }
`)

  writeFileSync(join(projectDir, '.gitignore'), 'node_modules\n.nuxt\n.output\ndist\n.DS_Store\n')
  writeFileSync(join(projectDir, '.prettierrc'), JSON.stringify({ semi: false, singleQuote: true, tabWidth: 2, trailingComma: 'none', printWidth: 100 }))
  writeFileSync(join(projectDir, 'README.md'), `# ${name}\n\nNuxt 3 Application\n\nnpm run dev\n`)
}
