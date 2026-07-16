import { readFileSync, statSync, writeFileSync, mkdirSync } from 'fs'
import { join } from 'path'

export function createAdapter() {
  return {
    name: 'lit',
    detect(dir) {
      try {
        const pkg = readPackageJson(dir)
        return pkg?.dependencies?.lit || pkg?.devDependencies?.lit
      } catch { return false }
    },
    supportedVersions: ['3.0', '3.1', '3.2'],
    defaultVersion: '3.2',
    kind: 'Frontend',

    async dev(dir, port) {
      const { spawn } = await import('child_process')
      const proc = spawn('npx', ['@web/dev-server', '--port', String(port || 8000)], { cwd: dir, stdio: 'inherit' })
      return new Promise((resolve, reject) => {
        proc.on('close', (code) => code === 0 ? resolve() : reject(new Error(`Dev server exited with code ${code}`)))
      })
    },

    async build(dir) {
      const { execFile } = await import('child_process')
      await execFile('npx', ['@web/rollup'], { cwd: dir })
    },

    async test(dir) {
      const { execFile } = await import('child_process')
      await execFile('npx', ['@web/test-runner'], { cwd: dir })
    },

    async lint(dir) {
      const { execFile } = await import('child_process')
      await execFile('npx', ['eslint', '.'], { cwd: dir })
    },

    async format(dir, write) {
      const { execFile } = await import('child_process')
      await execFile('npx', write ? ['prettier', '--write', '.'] : ['prettier', '--check', '.'], { cwd: dir })
    },

    scaffold(name, options) {
      return scaffoldLit(name, options)
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

async function scaffoldLit(name, options) {
  const projectDir = join(options.dir || process.cwd(), name)
  mkdirSync(join(projectDir, 'src', 'components'), { recursive: true })

  writeFileSync(join(projectDir, 'package.json'), JSON.stringify({
    name, version: '1.0.0', private: true, type: 'module',
    scripts: {
      dev: 'web-dev-server --port 8000',
      build: 'rollup -c',
      test: 'web-test-runner',
      lint: 'eslint .',
      format: 'prettier --write .'
    },
    dependencies: { lit: '^3.2.0' },
    devDependencies: { '@web/dev-server': '^0.4.0', '@web/rollup': '^0.6.0', '@web/test-runner': '^0.18.0', eslint: '^9.20.0', prettier: '^3.5.0' }
  }, null, 2))

  writeFileSync(join(projectDir, 'src', 'components', 'my-element.js'), `import { LitElement, html, css } from 'lit'

class MyElement extends LitElement {
  static styles = css\`
    :host {
      display: block;
      padding: 1rem;
      color: #333;
    }
    h1 { font-size: 1.5rem; }
  \`

  static properties = {
    name: { type: String }
  }

  constructor() {
    super()
    this.name = 'World'
  }

  render() {
    return html\`<h1>Hello, \${this.name}!</h1>\`
  }
}

customElements.define('my-element', MyElement)
`)

  writeFileSync(join(projectDir, 'index.html'), `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>${name}</title>
  <script type="module" src="/src/components/my-element.js"></script>
</head>
<body>
  <my-element name="${name}"></my-element>
</body>
</html>
`)

  writeFileSync(join(projectDir, 'web-dev-server.config.js'), `export default {
  open: true,
  watch: true,
  nodeResolve: true,
  appIndex: 'index.html'
}
`)

  writeFileSync(join(projectDir, '.gitignore'), 'node_modules\n.DS_Store\ndist\n')
  writeFileSync(join(projectDir, '.prettierrc'), JSON.stringify({ semi: true, singleQuote: true, tabWidth: 2, trailingComma: 'es5', printWidth: 100 }))
  writeFileSync(join(projectDir, 'README.md'), `# ${name}

Lit app scaffolded by Klyron

## Getting Started

npm run dev
`)
}
