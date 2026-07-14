import { readFileSync, statSync, writeFileSync, mkdirSync } from 'fs'
import { join } from 'path'

export function createAdapter() {
  return {
    name: 'astro',
    detect(dir) {
      return existsSync(join(dir, 'astro.config.mjs')) || existsSync(join(dir, 'astro.config.ts')) || existsSync(join(dir, 'astro.config.js'))
    },
    supportedVersions: ['4.0', '5.0', '5.4'],
    defaultVersion: '5.4',
    kind: 'StaticSiteGenerator',

    async dev(dir, port) {
      const { spawn } = await import('child_process')
      const proc = spawn('npx', ['astro', 'dev'], { cwd: dir, env: { ...process.env, PORT: String(port || 4321) }, stdio: 'inherit' })
      return new Promise((resolve, reject) => {
        proc.on('close', (code) => code === 0 ? resolve() : reject(new Error(`Dev server exited with code ${code}`)))
      })
    },

    async build(dir) {
      const { execFile } = await import('child_process')
      await execFile('npx', ['astro', 'build'], { cwd: dir })
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
      return scaffoldAstro(name, options)
    }
  }
}

function existsSync(p) {
  try { return statSync(p).isFile() } catch { return false }
}

async function scaffoldAstro(name, options) {
  const projectDir = join(options.dir || process.cwd(), name)
  mkdirSync(join(projectDir, 'src', 'pages'), { recursive: true })
  mkdirSync(join(projectDir, 'src', 'components'), { recursive: true })
  mkdirSync(join(projectDir, 'src', 'layouts'), { recursive: true })
  mkdirSync(join(projectDir, 'public'), { recursive: true })

  writeFileSync(join(projectDir, 'package.json'), JSON.stringify({
    name, version: '1.0.0', private: true, type: 'module',
    scripts: { dev: 'astro dev', build: 'astro build', preview: 'astro preview', test: 'vitest run', lint: 'eslint .', format: 'prettier --write .' },
    dependencies: { astro: '^5.4.0' },
    devDependencies: { '@astrojs/check': '^0.9.0', typescript: '^5.7.0', vitest: '^3.0.0', eslint: '^9.20.0', prettier: '^3.5.0', 'prettier-plugin-astro': '^0.14.0' }
  }, null, 2))

  writeFileSync(join(projectDir, 'astro.config.mjs'), `import { defineConfig } from 'astro/config'

export default defineConfig({
  site: 'https://example.com',
})
`)

  writeFileSync(join(projectDir, 'tsconfig.json'), JSON.stringify({
    compilerOptions: {
      target: 'ES2022', module: 'ESNext', moduleResolution: 'bundler',
      strict: true, skipLibCheck: true, jsx: 'preserve',
      allowImportingTsExtensions: true, noEmit: true
    },
    include: ['src']
  }, null, 2))

  writeFileSync(join(projectDir, 'src', 'layouts', 'Layout.astro'), `---
interface Props {
  title?: string
}
const { title } = Astro.props
---
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>{title || '${name}'}</title>
  </head>
  <body>
    <slot />
  </body>
</html>
`)

  writeFileSync(join(projectDir, 'src', 'pages', 'index.astro'), `---
import Layout from '../layouts/Layout.astro'
---

<Layout title="${name}">
  <main>
    <h1>Welcome to ${name}</h1>
    <p>Built with Astro</p>
  </main>
</Layout>
`)

  writeFileSync(join(projectDir, 'src', 'components', 'Header.astro'), `--- ---
<header>
  <nav>
    <a href="/">Home</a>
  </nav>
</header>
`)

  writeFileSync(join(projectDir, 'public', 'favicon.svg'), `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32"><circle cx="16" cy="16" r="16" fill="#646CFF"/></svg>`)
  writeFileSync(join(projectDir, '.gitignore'), 'node_modules\ndist\n.DS_Store\n')
  writeFileSync(join(projectDir, '.prettierrc'), JSON.stringify({ semi: false, singleQuote: true, tabWidth: 2, trailingComma: 'none', printWidth: 100 }))
  writeFileSync(join(projectDir, 'README.md'), `# ${name}\n\nAstro site\n\nnpm run dev\n`)
}
