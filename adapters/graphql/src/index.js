import { readFileSync, statSync, writeFileSync, mkdirSync } from 'fs'
import { join } from 'path'

export function createAdapter() {
  return {
    name: 'graphql',
    detect(dir) {
      try {
        const pkg = readPackageJson(dir)
        return pkg?.dependencies?.graphql || pkg?.devDependencies?.graphql ||
               pkg?.dependencies?.['@graphql-yoga'] || pkg?.devDependencies?.['@graphql-yoga']
      } catch { return false }
    },
    supportedVersions: ['16.9', '17.0'],
    defaultVersion: '17.0',
    kind: 'ApiFramework',

    async dev(dir, port) {
      const { spawn } = await import('child_process')
      const proc = spawn('npx', ['tsx', 'watch', 'src/index.ts'], { cwd: dir, env: { ...process.env, PORT: String(port || 4000) }, stdio: 'inherit' })
      return new Promise((resolve, reject) => {
        proc.on('close', (code) => code === 0 ? resolve() : reject(new Error(`Dev server exited with code ${code}`)))
      })
    },

    async build(dir) {
      const { execFile } = await import('child_process')
      await execFile('npx', ['tsc'], { cwd: dir })
    },

    async test(dir) {
      const { execFile } = await import('child_process')
      await execFile('npx', ['jest'], { cwd: dir })
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
      return scaffoldGraphql(name, options)
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

async function scaffoldGraphql(name, options) {
  const projectDir = join(options.dir || process.cwd(), name)
  mkdirSync(join(projectDir, 'src', 'schema'), { recursive: true })
  mkdirSync(join(projectDir, 'src', 'resolvers'), { recursive: true })

  writeFileSync(join(projectDir, 'package.json'), JSON.stringify({
    name, version: '1.0.0', private: true, type: 'module',
    scripts: {
      dev: 'tsx watch src/index.ts',
      build: 'tsc',
      test: 'jest',
      lint: 'eslint .',
      format: 'prettier --write .'
    },
    dependencies: { graphql: '^17.0.0', '@graphql-yoga': '^5.0.0' },
    devDependencies: { typescript: '^5.6.0', tsx: '^4.19.0', jest: '^30.0.0', eslint: '^9.20.0', prettier: '^3.5.0' }
  }, null, 2))

  writeFileSync(join(projectDir, 'tsconfig.json'), JSON.stringify({
    compilerOptions: {
      target: 'ES2022', module: 'ES2022', moduleResolution: 'bundler',
      strict: true, esModuleInterop: true, outDir: 'dist',
      rootDir: 'src', declaration: true
    },
    include: ['src']
  }, null, 2))

  writeFileSync(join(projectDir, 'src', 'index.ts'), `import { createServer } from '@graphql-yoga'
import { schema } from './schema/index.js'

const port = process.env.PORT || 4000

async function main() {
  const server = createServer({ schema })
  await server.start(port)
  console.log(\`GraphQL server running on http://localhost:\${port}/graphql\`)
}

main()
`)

  writeFileSync(join(projectDir, 'src', 'schema', 'index.ts'), `import { buildSchema } from 'graphql'

export const schema = buildSchema(\`
  type Query {
    hello(name: String): String!
    health: Health!
  }

  type Health {
    status: String!
    timestamp: String!
  }
\`)
`)

  writeFileSync(join(projectDir, 'src', 'resolvers', 'index.ts'), `export const resolvers = {
  Query: {
    hello: (_, { name }) => \`Hello, \${name || 'World'}!\`,
    health: () => ({ status: 'ok', timestamp: new Date().toISOString() })
  }
}
`)

  writeFileSync(join(projectDir, '.gitignore'), 'node_modules\n.DS_Store\ndist\n')
  writeFileSync(join(projectDir, '.prettierrc'), JSON.stringify({ semi: true, singleQuote: true, tabWidth: 2, trailingComma: 'es5', printWidth: 100 }))
  writeFileSync(join(projectDir, 'README.md'), `# ${name}

GraphQL API

## Getting Started

npm run dev
`)
}
