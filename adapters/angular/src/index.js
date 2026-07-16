import { readFileSync, statSync, writeFileSync, mkdirSync } from 'fs'
import { join } from 'path'

export function createAdapter() {
  return {
    name: 'angular',
    detect(dir) {
      try {
        const pkg = readPackageJson(dir)
        return pkg?.dependencies?.['@angular/core'] || pkg?.devDependencies?.['@angular/core'] || existsSync(join(dir, 'angular.json'))
      } catch { return false }
    },
    supportedVersions: ['17.0', '18.0', '19.0'],
    defaultVersion: '19.0',
    kind: 'Frontend',

    async dev(dir, port) {
      const { spawn } = await import('child_process')
      const proc = spawn('npx', ['ng', 'serve'], {
        cwd: dir,
        env: { ...process.env, PORT: String(port || 4200) },
        stdio: 'inherit'
      })
      return new Promise((resolve, reject) => {
        proc.on('close', (code) => code === 0 ? resolve() : reject(new Error(`Dev server exited with code ${code}`)))
      })
    },

    async build(dir) {
      const { execFile } = await import('child_process')
      await execFile('npx', ['ng', 'build'], { cwd: dir })
    },

    async test(dir) {
      const { execFile } = await import('child_process')
      await execFile('npx', ['ng', 'test'], { cwd: dir })
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
      return scaffoldAngular(name, options)
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

async function scaffoldAngular(name, options) {
  const projectDir = join(options.dir || process.cwd(), name)
  mkdirSync(join(projectDir, 'src', 'app'), { recursive: true })
  mkdirSync(join(projectDir, 'src', 'assets'), { recursive: true })
  mkdirSync(join(projectDir, 'src', 'environments'), { recursive: true })
  mkdirSync(join(projectDir, 'public'), { recursive: true })

  writeFileSync(join(projectDir, 'package.json'), JSON.stringify({
    name, version: '1.0.0', private: true,
    scripts: { ng: 'ng', start: 'ng serve', build: 'ng build', watch: 'ng build --watch --configuration development', test: 'ng test', lint: 'eslint .', format: 'prettier --write .' },
    dependencies: { '@angular/animations': '^19.0.0', '@angular/common': '^19.0.0', '@angular/compiler': '^19.0.0', '@angular/core': '^19.0.0', '@angular/forms': '^19.0.0', '@angular/platform-browser': '^19.0.0', '@angular/platform-browser-dynamic': '^19.0.0', '@angular/router': '^19.0.0', rxjs: '^7.8.0', 'tslib': '^2.8.0', 'zone.js': '~0.15.0' },
    devDependencies: { '@angular-devkit/build-angular': '^19.0.0', '@angular/cli': '^19.0.0', '@angular/compiler-cli': '^19.0.0', typescript: '~5.7.0', '@types/jasmine': '~5.1.0', jasmine: '~5.6.0', 'jasmine-core': '~5.6.0', 'karma': '~6.4.0', 'karma-chrome-launcher': '~3.2.0', 'karma-coverage': '~2.2.0', 'karma-jasmine': '~5.1.0', 'karma-jasmine-html-reporter': '~2.1.0', eslint: '^9.20.0', '@eslint/js': '^9.20.0', prettier: '^3.5.0' }
  }, null, 2))

  writeFileSync(join(projectDir, 'tsconfig.json'), JSON.stringify({
    compileOnSave: false,
    compilerOptions: {
      baseUrl: './', outDir: './dist', forceConsistentCasingInFileNames: true, strict: true, noImplicitOverride: true,
      noPropertyAccessFromIndexSignature: true, noImplicitReturns: true, noFallthroughCasesInSwitch: true,
      sourceMap: true, declaration: false, downlevelIteration: true, experimentalDecorators: true,
      moduleResolution: 'bundler', importHelpers: true, target: 'ES2022', module: 'ES2022', lib: ['ES2022', 'dom'],
      skipLibCheck: true
    },
    include: ['src/**/*.ts'],
    exclude: ['node_modules', 'dist']
  }, null, 2))

  writeFileSync(join(projectDir, 'tsconfig.app.json'), JSON.stringify({
    extends: './tsconfig.json',
    compilerOptions: { outDir: './out-tsc/app', types: [] },
    files: ['src/main.ts'],
    include: ['src/**/*.d.ts']
  }, null, 2))

  writeFileSync(join(projectDir, 'angular.json'), JSON.stringify({
    $schema: './node_modules/@angular/cli/lib/config/schema.json',
    version: 1,
    newProjectRoot: 'projects',
    projects: {
      [name]: {
        projectType: 'application',
        schematics: {},
        root: '',
        sourceRoot: 'src',
        prefix: 'app',
        architect: {
          build: {
            builder: '@angular-devkit/build-angular:application',
            options: {
              outputPath: { base: 'dist' },
              index: 'src/index.html',
              browser: 'src/main.ts',
              polyfills: ['zone.js'],
              tsConfig: 'tsconfig.app.json',
              assets: [{ glob: '**/*', input: 'public' }],
              styles: ['src/styles.css'],
              scripts: []
            },
            configurations: { production: { budgets: [{ type: 'initial', maximumWarning: '500kb', maximumError: '1mb' }], outputHashing: 'all' }, development: { buildOptimizer: false, optimization: false, vendorChunk: true, extractLicenses: false, sourceMap: true, namedChunks: true } },
            defaultConfiguration: 'production'
          },
          serve: { builder: '@angular-devkit/build-angular:dev-server', configurations: { production: { buildTarget: `${name}:build:production` }, development: { buildTarget: `${name}:build:development` } }, defaultConfiguration: 'development' },
          test: { builder: '@angular-devkit/build-angular:karma', options: { polyfills: ['zone.js', 'zone.js/testing'], tsConfig: 'tsconfig.spec.json', assets: [{ glob: '**/*', input: 'public' }], styles: ['src/styles.css'], scripts: [] } }
        }
      }
    }
  }, null, 2))

  writeFileSync(join(projectDir, 'src', 'index.html'), `<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <title>${name}</title>
    <base href="/" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <link rel="icon" type="image/x-icon" href="favicon.ico" />
  </head>
  <body>
    <app-root></app-root>
  </body>
</html>
`)

  writeFileSync(join(projectDir, 'src', 'main.ts'), `import { bootstrapApplication } from '@angular/platform-browser'
import { AppComponent } from './app/app.component'
import { provideRouter } from '@angular/router'
import { routes } from './app/app.routes'

bootstrapApplication(AppComponent, {
  providers: [provideRouter(routes)]
}).catch((err) => console.error(err))
`)

  writeFileSync(join(projectDir, 'src', 'styles.css'), `* { margin: 0; padding: 0; box-sizing: border-box; }
:root { font-family: Inter, system-ui, Avenir, Helvetica, Arial, sans-serif; line-height: 1.5; color: #213547; background-color: #fff; }
body { min-height: 100vh; }
h1 { font-size: 3.2em; line-height: 1.1; }
`)

  writeFileSync(join(projectDir, 'src', 'app', 'app.component.ts'), `import { Component } from '@angular/core'
import { RouterOutlet } from '@angular/router'

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [RouterOutlet],
  template: '<router-outlet></router-outlet>'
})
export class AppComponent {}
`)

  writeFileSync(join(projectDir, 'src', 'app', 'app.routes.ts'), `import { Routes } from '@angular/router'

export const routes: Routes = [
  { path: '', loadComponent: () => import('./home/home.component').then(m => m.HomeComponent) }
]
`)

  writeFileSync(join(projectDir, 'src', 'app', 'app.config.ts'), `import { ApplicationConfig, provideZoneChangeDetection } from '@angular/core'
import { provideRouter } from '@angular/router'
import { routes } from './app.routes'

export const appConfig: ApplicationConfig = {
  providers: [provideZoneChangeDetection({ eventCoalescing: true }), provideRouter(routes)]
}
`)

  mkdirSync(join(projectDir, 'src', 'app', 'home'), { recursive: true })
  writeFileSync(join(projectDir, 'src', 'app', 'home', 'home.component.ts'), `import { Component } from '@angular/core'

@Component({
  selector: 'app-home',
  standalone: true,
  template: '<h1>Welcome to ${name}</h1><p>Built with Angular</p>'
})
export class HomeComponent {}
`)

  writeFileSync(join(projectDir, '.gitignore'), 'node_modules\ndist\n.DS_Store\n*.tsbuildinfo\n')
  writeFileSync(join(projectDir, '.prettierrc'), JSON.stringify({ semi: true, singleQuote: true, tabWidth: 2, trailingComma: 'es5', printWidth: 100 }))
  writeFileSync(join(projectDir, 'eslint.config.js'), `import js from '@eslint/js'
export default [js.configs.recommended, { ignores: ['node_modules', 'dist'] }]
`)
  writeFileSync(join(projectDir, 'README.md'), `# ${name}\n\nAngular App\n\nnpm start\n`)
}
