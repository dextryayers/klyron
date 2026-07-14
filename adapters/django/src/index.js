import { readFileSync, statSync, writeFileSync, mkdirSync } from 'fs'
import { join } from 'path'

export function createAdapter() {
  return {
    name: 'django',
    detect(dir) {
      return existsSync(join(dir, 'manage.py'))
    },
    supportedVersions: ['4.2', '5.0'],
    defaultVersion: '5.0',
    kind: 'Polyglot',

    async dev(dir, port) {
      const { spawn } = await import('child_process')
      const args = ['manage.py', 'runserver']
      if (port) args.push(`0.0.0.0:${port}`)
      const proc = spawn('python3', args, { cwd: dir, stdio: 'inherit' })
      return new Promise((resolve, reject) => {
        proc.on('close', (code) => code === 0 ? resolve() : reject(new Error(`Dev server exited with code ${code}`)))
      })
    },

    async build(dir) {
      const { execFile } = await import('child_process')
      await execFile('python3', ['manage.py', 'collectstatic', '--noinput'], { cwd: dir })
    },

    async test(dir) {
      const { execFile } = await import('child_process')
      await execFile('python3', ['manage.py', 'test'], { cwd: dir })
    },

    async lint(dir) {
      const { execFile } = await import('child_process')
      if (existsSync(join(dir, '.flake8')) || existsSync(join(dir, 'setup.cfg'))) {
        await execFile('python3', ['-m', 'flake8'], { cwd: dir })
      }
    },

    async format(dir, writeMode) {
      const { execFile } = await import('child_process')
      const args = writeMode ? ['-m', 'black', '.'] : ['-m', 'black', '--check', '.']
      await execFile('python3', args, { cwd: dir })
    },

    scaffold(name, options) {
      return scaffoldDjango(name, options)
    }
  }
}

function existsSync(p) {
  try { return statSync(p).isFile() } catch { return false }
}

async function scaffoldDjango(name, options) {
  const projectDir = join(options.dir || process.cwd(), name)
  const projectName = name.replace(/[^a-zA-Z0-9_]/g, '_').toLowerCase()

  writeFileSync(join(projectDir, 'requirements.txt'), `Django>=5.0,<5.1
django-cors-headers>=4.0
djangorestframework>=3.15
python-decouple>=3.8
`)
  writeFileSync(join(projectDir, '.env'), `DEBUG=True
SECRET_KEY=change-me-to-a-random-secret-key
ALLOWED_HOSTS=localhost,127.0.0.1
`)
  writeFileSync(join(projectDir, 'README.md'), `# ${name}\n\nDjango application\n\npython manage.py runserver\n`)
}
