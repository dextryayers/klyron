import { readFileSync, statSync, writeFileSync, mkdirSync } from 'fs'
import { join } from 'path'

export function createAdapter() {
  return {
    name: 'nestjs',
    detect(dir) {
      return existsSync(join(dir, 'nest-cli.json')) || existsSync(join(dir, 'nest.config.json'))
    },
    supportedVersions: ['10.0'],
    defaultVersion: '10.0',
    kind: 'Backend',

    async dev(dir, port) {
      const { spawn } = await import('child_process')
      const proc = spawn('npx', ['nest', 'start', '--watch'], { cwd: dir, env: { ...process.env, PORT: String(port || 3000) }, stdio: 'inherit' })
      return new Promise((resolve, reject) => {
        proc.on('close', (code) => code === 0 ? resolve() : reject(new Error(`Dev server exited with code ${code}`)))
      })
    },

    async build(dir) {
      const { execFile } = await import('child_process')
      await execFile('npx', ['nest', 'build'], { cwd: dir })
    },

    async test(dir) {
      const { execFile } = await import('child_process')
      await execFile('npx', ['jest'], { cwd: dir })
    },

    async lint(dir) {
      const { execFile } = await import('child_process')
      await execFile('npx', ['eslint', '{src,apps,libs,test}/**/*.ts'], { cwd: dir })
    },

    async format(dir, writeMode) {
      const { execFile } = await import('child_process')
      await execFile('npx', writeMode ? ['prettier', '--write', '.'] : ['prettier', '--check', '.'], { cwd: dir })
    },

    scaffold(name, options) {
      return scaffoldNest(name, options)
    }
  }
}

function existsSync(p) {
  try { return statSync(p).isFile() } catch { return false }
}

async function scaffoldNest(name, options) {
  const projectDir = join(options.dir || process.cwd(), name)
  mkdirSync(join(projectDir, 'src'), { recursive: true })
  mkdirSync(join(projectDir, 'test'), { recursive: true })

  writeFileSync(join(projectDir, 'package.json'), JSON.stringify({
    name, version: '1.0.0', private: true,
    scripts: {
      build: 'nest build', start: 'nest start', dev: 'nest start --watch',
      startDebug: 'nest start --debug --watch', startProd: 'node dist/main',
      test: 'jest', testE2E: 'jest --config ./test/jest-e2e.json',
      lint: 'eslint {src,apps,libs,test}/**/*.ts --fix', format: 'prettier --write "src/**/*.ts" "test/**/*.ts"'
    },
    dependencies: {
      '@nestjs/common': '^10.0.0', '@nestjs/core': '^10.0.0', '@nestjs/platform-express': '^10.0.0',
      'reflect-metadata': '^0.2.0', 'rxjs': '^7.8.0', 'class-validator': '^0.14.0', 'class-transformer': '^0.5.1'
    },
    devDependencies: {
      '@nestjs/cli': '^10.0.0', '@nestjs/schematics': '^10.0.0', '@nestjs/testing': '^10.0.0',
      typescript: '^5.7.0', '@types/node': '^22.13.0', '@types/express': '^5.0.0',
      jest: '^30.0.0', 'ts-jest': '^29.2.0', '@types/jest': '^30.0.0',
      eslint: '^9.20.0', '@typescript-eslint/eslint-plugin': '^8.24.0', '@typescript-eslint/parser': '^8.24.0',
      prettier: '^3.5.0', 'ts-loader': '^9.5.0', 'ts-node': '^10.9.0'
    },
    jest: {
      moduleFileExtensions: ['js', 'json', 'ts'], rootDir: 'src', testRegex: '.*\\.spec\\.ts$',
      transform: { '^.+\\.(t|j)s$': 'ts-jest' }, collectCoverageFrom: ['**/*.(t|j)s'],
      coverageDirectory: '../coverage', testEnvironment: 'node'
    }
  }, null, 2))

  writeFileSync(join(projectDir, 'tsconfig.json'), JSON.stringify({
    compilerOptions: {
      module: 'commonjs', declaration: true, removeComments: true, emitDecoratorMetadata: true,
      experimentalDecorators: true, allowSyntheticDefaultImports: true, target: 'ES2022',
      sourceMap: true, outDir: './dist', baseUrl: './', incremental: true, skipLibCheck: true,
      strictNullChecks: true, noImplicitAny: true, strictBindCallApply: true,
      forceConsistentCasingInFileNames: true, noFallthroughCasesInSwitch: true
    }
  }, null, 2))

  writeFileSync(join(projectDir, 'tsconfig.build.json'), JSON.stringify({
    extends: './tsconfig.json', exclude: ['node_modules', 'test', 'dist', '**/*spec.ts']
  }, null, 2))

  writeFileSync(join(projectDir, 'nest-cli.json'), JSON.stringify({
    $schema: 'https://json.schemastore.org/nest-cli',
    collection: '@nestjs/schematics', sourceRoot: 'src',
    compilerOptions: { deleteOutDir: true }
  }, null, 2))

  writeFileSync(join(projectDir, 'src', 'main.ts'), `import { NestFactory } from '@nestjs/core'
import { ValidationPipe } from '@nestjs/common'
import { AppModule } from './app.module'

async function bootstrap() {
  const app = await NestFactory.create(AppModule)
  app.useGlobalPipes(new ValidationPipe({ whitelist: true, transform: true }))
  app.enableCors()
  await app.listen(process.env.PORT || 3000)
  console.log(\`${name} running on http://localhost:\${process.env.PORT || 3000}\`)
}
bootstrap()
`)

  writeFileSync(join(projectDir, 'src', 'app.module.ts'), `import { Module } from '@nestjs/common'
import { AppController } from './app.controller'
import { AppService } from './app.service'

@Module({
  imports: [],
  controllers: [AppController],
  providers: [AppService],
})
export class AppModule {}
`)

  writeFileSync(join(projectDir, 'src', 'app.controller.ts'), `import { Controller, Get } from '@nestjs/common'
import { AppService } from './app.service'

@Controller()
export class AppController {
  constructor(private readonly appService: AppService) {}

  @Get()
  getHello(): string {
    return this.appService.getHello()
  }
}
`)

  writeFileSync(join(projectDir, 'src', 'app.service.ts'), `import { Injectable } from '@nestjs/common'

@Injectable()
export class AppService {
  getHello(): string {
    return 'Hello from ${name}!'
  }
}
`)

  writeFileSync(join(projectDir, 'src', 'app.controller.spec.ts'), `import { Test, TestingModule } from '@nestjs/testing'
import { AppController } from './app.controller'
import { AppService } from './app.service'

describe('AppController', () => {
  let appController: AppController

  beforeEach(async () => {
    const app: TestingModule = await Test.createTestingModule({
      controllers: [AppController],
      providers: [AppService],
    }).compile()

    appController = app.get<AppController>(AppController)
  })

  describe('root', () => {
    it('should return "Hello from ${name}!"', () => {
      expect(appController.getHello()).toBe('Hello from ${name}!')
    })
  })
})
`)

  writeFileSync(join(projectDir, '.eslintrc.js'), `module.exports = {
  parser: '@typescript-eslint/parser',
  parserOptions: { project: 'tsconfig.json', tsconfigRootDir: __dirname, sourceType: 'module' },
  plugins: ['@typescript-eslint/eslint-plugin'],
  extends: ['plugin:@typescript-eslint/recommended', 'plugin:prettier/recommended'],
  root: true, env: { node: true, jest: true },
  ignorePatterns: ['.eslintrc.js'],
  rules: { '@typescript-eslint/interface-name-prefix': 'off', '@typescript-eslint/explicit-function-return-type': 'off', '@typescript-eslint/explicit-module-boundary-types': 'off', '@typescript-eslint/no-explicit-any': 'off' },
}
`)

  writeFileSync(join(projectDir, '.prettierrc'), JSON.stringify({ singleQuote: true, trailingComma: 'all', tabWidth: 2, semi: false, printWidth: 120 }))
  writeFileSync(join(projectDir, '.gitignore'), 'node_modules\ndist\n.DS_Store\n*.js.map\n')
  writeFileSync(join(projectDir, 'README.md'), `# ${name}\n\nNestJS application\n\nnpm run dev\n`)
}
