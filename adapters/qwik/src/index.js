import { readFileSync, statSync, writeFileSync, mkdirSync } from 'fs'
import { join } from 'path'

export function createAdapter() {
  return {
    name: 'qwik',
    detect(dir) {
      try {
        const pkg = readPackageJson(dir)
        return pkg?.dependencies?.['@builder.io/qwik'] || pkg?.devDependencies?.['@builder.io/qwik']
      } catch { return false }
    },
    supportedVersions: ['1.9'],
    defaultVersion: '1.9',
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
      return scaffoldQwik(name, options)
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

async function scaffoldQwik(name, options) {
  const projectDir = join(options.dir || process.cwd(), name)
  mkdirSync(join(projectDir, 'src', 'routes'), { recursive: true })
  mkdirSync(join(projectDir, 'src', 'components'), { recursive: true })
  mkdirSync(join(projectDir, 'public'), { recursive: true })

  writeFileSync(join(projectDir, 'package.json'), JSON.stringify({
    name, version: '1.0.0', private: true, type: 'module',
    scripts: { dev: 'vite', build: 'vite build', preview: 'vite preview', test: 'vitest run', lint: 'eslint .', format: 'prettier --write .', 'qwik.build': 'qwik build', 'qwik.preview': 'qwik preview' },
    dependencies: { '@builder.io/qwik': '^1.12.0', '@builder.io/qwik-city': '^1.12.0' },
    devDependencies: { vite: '^6.1.0', 'vite-plugin-qwik': '^1.12.0', typescript: '^5.7.0', vitest: '^3.0.0', eslint: '^9.20.0', '@eslint/js': '^9.20.0', prettier: '^3.5.0' }
  }, null, 2))

  writeFileSync(join(projectDir, 'tsconfig.json'), JSON.stringify({
    compilerOptions: {
      target: 'ES2022', module: 'ESNext', moduleResolution: 'bundler', strict: true, jsx: 'react-jsx',
      jsxImportSource: '@builder.io/qwik', skipLibCheck: true, noEmit: true, isolatedModules: true,
      resolveJsonModule: true
    },
    include: ['src']
  }, null, 2))

  writeFileSync(join(projectDir, 'vite.config.ts'), `import { defineConfig } from 'vite'
import { qwikVite } from '@builder.io/qwik/optimizer'
import { qwikCity } from '@builder.io/qwik-city/vite'

export default defineConfig({
  plugins: [qwikCity(), qwikVite()],
  server: { port: 5173, host: true },
})
`)

  writeFileSync(join(projectDir, 'src', 'routes', 'index.tsx'), `import { component$ } from '@builder.io/qwik'
import { routeLoader$ } from '@builder.io/qwik-city'

export const useServerTime = routeLoader$(() => ({ date: new Date().toISOString() }))

export default component$(() => {
  const serverTime = useServerTime()
  return (
    <div>
      <h1>Welcome to ${name}</h1>
      <p>Built with Qwik</p>
      <p>Server time: {serverTime.value.date}</p>
    </div>
  )
})
`)

  writeFileSync(join(projectDir, 'src', 'routes', 'layout.tsx'), `import { component$, Slot } from '@builder.io/qwik'

export default component$(() => {
  return <Slot />
})
`)

  writeFileSync(join(projectDir, 'src', 'entry.ssr.tsx'), `import { renderToStream, RenderToStreamOptions } from '@builder.io/qwik/server'
import { manifest } from '@qwik-client-manifest'
import Root from './root'

export default function (opts: RenderToStreamOptions) {
  return renderToStream(<Root />, { manifest, ...opts })
}
`)

  writeFileSync(join(projectDir, 'src', 'entry.dev.tsx'), `import { render } from '@builder.io/qwik'
import Root from './root'

render(document, <Root />)
`)

  writeFileSync(join(projectDir, 'src', 'root.tsx'), `import { component$ } from '@builder.io/qwik'
import { QwikCityProvider, RouterOutlet, ServiceWorkerRegister } from '@builder.io/qwik-city'

export default component$(() => {
  return (
    <QwikCityProvider>
      <head>
        <meta charset="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
      </head>
      <body>
        <RouterOutlet />
        <ServiceWorkerRegister />
      </body>
    </QwikCityProvider>
  )
})
`)

  writeFileSync(join(projectDir, 'src', 'global.css'), `* { margin: 0; padding: 0; box-sizing: border-box; }
:root { font-family: Inter, system-ui, Avenir, Helvetica, Arial, sans-serif; line-height: 1.5; color: #213547; background-color: #fff; }
body { min-height: 100vh; }
h1 { font-size: 3.2em; line-height: 1.1; }
`)

  writeFileSync(join(projectDir, '.gitignore'), 'node_modules\ndist\nserver\n.DS_Store\n*.tsbuildinfo\n')
  writeFileSync(join(projectDir, '.prettierrc'), JSON.stringify({ semi: true, singleQuote: true, tabWidth: 2, trailingComma: 'es5', printWidth: 100 }))
  writeFileSync(join(projectDir, 'eslint.config.js'), `import js from '@eslint/js'
export default [js.configs.recommended, { ignores: ['node_modules', 'dist', 'server'] }]
`)
  writeFileSync(join(projectDir, 'README.md'), `# ${name}\n\nQwik App\n\nnpm run dev\n`)
}
