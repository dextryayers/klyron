import { readFileSync, statSync, writeFileSync, mkdirSync } from 'fs'
import { join } from 'path'

export function createAdapter() {
  return {
    name: 'remix',
    detect(dir) {
      try {
        const pkg = readPackageJson(dir)
        return pkg?.dependencies?.['@remix-run/react'] || pkg?.devDependencies?.['@remix-run/react']
      } catch { return false }
    },
    supportedVersions: ['2.15'],
    defaultVersion: '2.15',
    kind: 'Fullstack',

    async dev(dir, port) {
      const { spawn } = await import('child_process')
      const proc = spawn('npx', ['remix', 'dev'], {
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
      await execFile('npx', ['remix', 'build'], { cwd: dir })
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
      return scaffoldRemix(name, options)
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

async function scaffoldRemix(name, options) {
  const projectDir = join(options.dir || process.cwd(), name)
  mkdirSync(join(projectDir, 'app', 'routes'), { recursive: true })
  mkdirSync(join(projectDir, 'app', 'styles'), { recursive: true })
  mkdirSync(join(projectDir, 'public'), { recursive: true })

  writeFileSync(join(projectDir, 'package.json'), JSON.stringify({
    name, version: '1.0.0', private: true,
    scripts: { dev: 'remix dev', build: 'remix build', start: 'remix-serve build/index.js', test: 'vitest run', lint: 'eslint .', format: 'prettier --write .' },
    dependencies: { '@remix-run/react': '^2.15.0', '@remix-run/node': '^2.15.0', '@remix-run/serve': '^2.15.0', isbot: '^5.1.0', react: '^18.3.0', 'react-dom': '^18.3.0' },
    devDependencies: { typescript: '^5.7.0', '@types/react': '^18.3.0', '@types/react-dom': '^18.3.0', vitest: '^3.0.0', eslint: '^9.20.0', '@eslint/js': '^9.20.0', prettier: '^3.5.0', '@remix-run/dev': '^2.15.0' }
  }, null, 2))

  writeFileSync(join(projectDir, 'remix.config.js'), `/** @type {import('@remix-run/dev').AppConfig} */
export default {
  ignoredRouteFiles: ['**/.*'],
  appDirectory: 'app',
  assetsBuildDirectory: 'public/build',
  serverBuildPath: 'build/index.js',
  publicPath: '/build/',
  future: { v2_errorBoundary: true, v2_meta: true, v2_normalizeFormMethod: true, v2_routeConvention: true }
}
`)

  writeFileSync(join(projectDir, 'tsconfig.json'), JSON.stringify({
    compilerOptions: {
      target: 'ES2022', lib: ['DOM', 'DOM.Iterable', 'ES2022'], module: 'ESNext', moduleResolution: 'bundler',
      resolveJsonModule: true, isolatedModules: true, noEmit: true, jsx: 'react-jsx', strict: true,
      forceConsistentCasingInFileNames: true, skipLibCheck: true
    },
    include: ['remix.env.d.ts', '**/*.ts', '**/*.tsx']
  }, null, 2))

  writeFileSync(join(projectDir, 'app', 'root.tsx'), `import { Links, Meta, Outlet, Scripts, ScrollRestoration } from '@remix-run/react'

export function links() {
  return [{ rel: 'stylesheet', href: '/app/styles/global.css' }]
}

export default function Root() {
  return (
    <html lang="en">
      <head><Meta /><Links /></head>
      <body>
        <Outlet />
        <ScrollRestoration />
        <Scripts />
      </body>
    </html>
  )
}
`)

  writeFileSync(join(projectDir, 'app', 'routes', '_index.tsx'), `export default function Index() {
  return (
    <div>
      <h1>Welcome to ${name}</h1>
      <p>Built with Remix</p>
    </div>
  )
}
`)

  writeFileSync(join(projectDir, 'app', 'entry.server.tsx'), `import { RemixServer } from '@remix-run/react'
import { handleRequest } from '@remix-run/node'
import { renderToString } from 'react-dom/server'

export default function handleRequest(request, responseStatusCode, responseHeaders, remixContext) {
  const html = renderToString(<RemixServer context={remixContext} url={request.url} />)
  responseHeaders.set('Content-Type', 'text/html')
  return new Response('<!DOCTYPE html>' + html, { status: responseStatusCode, headers: responseHeaders })
}
`)

  writeFileSync(join(projectDir, 'app', 'entry.client.tsx'), `import { RemixBrowser } from '@remix-run/react'
import { hydrateRoot } from 'react-dom/client'

hydrateRoot(document, <RemixBrowser />)
`)

  writeFileSync(join(projectDir, 'app', 'styles', 'global.css'), `* { margin: 0; padding: 0; box-sizing: border-box; }
:root { font-family: Inter, system-ui, Avenir, Helvetica, Arial, sans-serif; line-height: 1.5; color: #213547; background-color: #fff; }
body { min-height: 100vh; }
h1 { font-size: 3.2em; line-height: 1.1; }
`)

  writeFileSync(join(projectDir, 'remix.env.d.ts'), `/// <reference types="@remix-run/dev" />
/// <reference types="@remix-run/node" />
`)

  writeFileSync(join(projectDir, '.gitignore'), 'node_modules\n.build\npublic/build\n.DS_Store\n*.tsbuildinfo\n')
  writeFileSync(join(projectDir, '.prettierrc'), JSON.stringify({ semi: true, singleQuote: true, tabWidth: 2, trailingComma: 'es5', printWidth: 100 }))
  writeFileSync(join(projectDir, 'eslint.config.js'), `import js from '@eslint/js'
export default [js.configs.recommended, { ignores: ['node_modules', 'build', 'public/build'] }]
`)
  writeFileSync(join(projectDir, 'README.md'), `# ${name}\n\nRemix App\n\nnpm run dev\n`)
}
