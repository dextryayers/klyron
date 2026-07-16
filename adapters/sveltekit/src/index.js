import { readFileSync, statSync, writeFileSync, mkdirSync } from 'fs'
import { join } from 'path'

export function createAdapter() {
  return {
    name: 'sveltekit',
    detect(dir) {
      try {
        const pkg = readPackageJson(dir)
        return pkg?.dependencies?.['@sveltejs/kit'] || pkg?.devDependencies?.['@sveltejs/kit']
      } catch { return false }
    },
    supportedVersions: ['2.0'],
    defaultVersion: '2.0',
    kind: 'Fullstack',

    async dev(dir, port) {
      const { spawn } = await import('child_process')
      const proc = spawn('npx', ['vite', 'dev'], {
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
      return scaffoldSvelteKit(name, options)
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

async function scaffoldSvelteKit(name, options) {
  const projectDir = join(options.dir || process.cwd(), name)
  mkdirSync(join(projectDir, 'src', 'routes'), { recursive: true })
  mkdirSync(join(projectDir, 'src', 'lib'), { recursive: true })
  mkdirSync(join(projectDir, 'static'), { recursive: true })

  writeFileSync(join(projectDir, 'package.json'), JSON.stringify({
    name, version: '1.0.0', private: true,
    scripts: { dev: 'vite dev', build: 'vite build', preview: 'vite preview', test: 'vitest run', lint: 'eslint .', format: 'prettier --write .', check: 'svelte-kit sync && svelte-check' },
    dependencies: { '@sveltejs/kit': '^2.0.0', '@sveltejs/adapter-auto': '^4.0.0', svelte: '^5.0.0' },
    devDependencies: { '@sveltejs/vite-plugin-svelte': '^5.0.0', vite: '^6.1.0', typescript: '^5.7.0', vitest: '^3.0.0', eslint: '^9.20.0', '@eslint/js': '^9.20.0', prettier: '^3.5.0', 'prettier-plugin-svelte': '^3.3.0', 'svelte-check': '^4.1.0' }
  }, null, 2))

  writeFileSync(join(projectDir, 'svelte.config.js'), `import adapter from '@sveltejs/adapter-auto'
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte'

/** @type {import('@sveltejs/kit').Config} */
export default {
  preprocess: vitePreprocess(),
  kit: { adapter: adapter() }
}
`)

  writeFileSync(join(projectDir, 'vite.config.js'), `import { sveltekit } from '@sveltejs/kit/vite'
import { defineConfig } from 'vite'

export default defineConfig({ plugins: [sveltekit()] })
`)

  writeFileSync(join(projectDir, 'tsconfig.json'), JSON.stringify({
    extends: './.svelte-kit/tsconfig.json',
    compilerOptions: { strict: true, skipLibCheck: true }
  }, null, 2))

  writeFileSync(join(projectDir, 'src', 'app.html'), `<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width" />
    %sveltekit.head%
  </head>
  <body>
    <div style="display: contents">%sveltekit.body%</div>
  </body>
</html>
`)

  writeFileSync(join(projectDir, 'src', 'routes', '+layout.svelte'), `<script>
  import '../app.css'
</script>

<slot />
`)

  writeFileSync(join(projectDir, 'src', 'routes', '+page.svelte'), `<h1>Welcome to {name}</h1>
<p>Built with SvelteKit</p>

<script>
  let { data } = $props()
</script>
`)

  writeFileSync(join(projectDir, 'src', 'app.css'), `* { margin: 0; padding: 0; box-sizing: border-box; }
:root { font-family: Inter, system-ui, Avenir, Helvetica, Arial, sans-serif; line-height: 1.5; color: #213547; background-color: #fff; }
body { min-height: 100vh; }
h1 { font-size: 3.2em; line-height: 1.1; }
`)

  writeFileSync(join(projectDir, 'src', 'app.d.ts'), `/// <reference types="@sveltejs/kit" />
`)

  writeFileSync(join(projectDir, '.gitignore'), 'node_modules\n.svelte-kit\nbuild\n.DS_Store\n*.tsbuildinfo\n')
  writeFileSync(join(projectDir, '.prettierrc'), JSON.stringify({ semi: true, singleQuote: true, tabWidth: 2, trailingComma: 'es5', printWidth: 100, plugins: ['prettier-plugin-svelte'], overrides: [{ files: '*.svelte', options: { parser: 'svelte' } }] }))
  writeFileSync(join(projectDir, 'eslint.config.js'), `import js from '@eslint/js'
export default [js.configs.recommended, { ignores: ['node_modules', '.svelte-kit', 'build'] }]
`)
  writeFileSync(join(projectDir, 'README.md'), `# ${name}\n\nSvelteKit App\n\nnpm run dev\n`)
}
