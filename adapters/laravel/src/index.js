import { readFileSync, statSync, writeFileSync, mkdirSync } from 'fs'
import { join } from 'path'

export function createAdapter() {
  return {
    name: 'laravel',
    detect(dir) {
      return existsSync(join(dir, 'artisan'))
    },
    supportedVersions: ['9', '10', '11', '12', '13'],
    defaultVersion: '12',
    kind: 'Polyglot',

    async dev(dir, port) {
      const { spawn } = await import('child_process')
      const args = ['artisan', 'serve']
      if (port) args.push(`--port=${port}`)
      const proc = spawn('php', args, { cwd: dir, stdio: 'inherit' })
      return new Promise((resolve, reject) => {
        proc.on('close', (code) => code === 0 ? resolve() : reject(new Error(`Dev server exited with code ${code}`)))
      })
    },

    async build(dir) {
      const { execFile } = await import('child_process')
      await execFile('php', ['artisan', 'optimize'], { cwd: dir })
    },

    async test(dir) {
      const { execFile } = await import('child_process')
      await execFile('php', ['artisan', 'test'], { cwd: dir })
    },

    async lint(dir) {
      const { execFile } = await import('child_process')
      if (existsSync(join(dir, 'vendor/bin/pint'))) {
        await execFile('php', ['vendor/bin/pint', '--test'], { cwd: dir })
      }
    },

    async format(dir, writeMode) {
      const { execFile } = await import('child_process')
      if (existsSync(join(dir, 'vendor/bin/pint'))) {
        const args = writeMode ? ['vendor/bin/pint'] : ['vendor/bin/pint', '--test']
        await execFile('php', args, { cwd: dir })
      }
    },

    scaffold(name, options) {
      return scaffoldLaravel(name, options)
    }
  }
}

function existsSync(p) {
  try { return statSync(p).isFile() } catch { return false }
}

async function scaffoldLaravel(name, options) {
  const projectDir = join(options.dir || process.cwd(), name)
  mkdirSync(join(projectDir, 'app', 'Models'), { recursive: true })
  mkdirSync(join(projectDir, 'app', 'Http', 'Controllers'), { recursive: true })
  mkdirSync(join(projectDir, 'database', 'migrations'), { recursive: true })
  mkdirSync(join(projectDir, 'resources', 'views'), { recursive: true })
  mkdirSync(join(projectDir, 'routes'), { recursive: true })

  writeFileSync(join(projectDir, 'composer.json'), JSON.stringify({
    name: `${name}/${name}`, type: 'project', require: {
      php: '^8.2', 'laravel/framework': '^12.0', 'laravel/sanctum': '^4.0', 'laravel/tinker': '^2.9'
    },
    'require-dev': { 'laravel/pint': '^1.18', 'nunomaduro/collision': '^8.0', 'laravel/sail': '^1.40' },
    autoload: { 'psr-4': { 'App\\\\': 'app/', 'Database\\\\Factories\\\\': 'database/factories/', 'Database\\\\Seeders\\\\': 'database/seeders/' } },
    scripts: { 'post-autoload-dump': ['Illuminate\\\\Foundation\\\\ComposerScripts::postAutoloadDump', '@php artisan package:discover --ansi'] },
    extra: { 'laravel': { 'dont-discover': [] } }, config: { 'optimize-autoloader': true, 'preferred-install': 'dist', 'sort-packages': true }
  }, null, 2))

  writeFileSync(join(projectDir, '.env'), `APP_NAME="${name}"
APP_ENV=local
APP_KEY=
APP_DEBUG=true
APP_URL=http://localhost
DB_CONNECTION=sqlite
DB_DATABASE=database/database.sqlite
`)
  writeFileSync(join(projectDir, 'routes', 'web.php'), `<?php

use Illuminate\\Support\\Facades\\Route;

Route::get('/', function () {
    return view('welcome');
});
`)
  writeFileSync(join(projectDir, 'routes', 'api.php'), `<?php

use Illuminate\\Support\\Facades\\Route;

Route::get('/health', function () {
    return response()->json(['status' => 'ok', 'app' => '${name}']);
});
`)
  writeFileSync(join(projectDir, 'resources', 'views', 'welcome.blade.php'), `<!doctype html>
<html lang="en">
<head><meta charset="UTF-8"><meta name="viewport" content="width=device-width, initial-scale=1.0"><title>{{ config('app.name') }}</title></head>
<body>
  <h1>Welcome to <?php echo e(config('app.name')); ?></h1>
</body>
</html>
`)
  writeFileSync(join(projectDir, 'README.md'), `# ${name}\n\nLaravel application\n\nphp artisan serve\n`)
}
