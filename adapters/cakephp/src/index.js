import { readFileSync, statSync, writeFileSync, mkdirSync } from 'fs'
import { join } from 'path'

export function createAdapter() {
  return {
    name: 'CakePHP',
    detect(dir) {
      try {
        const composer = readFileSync(join(dir, 'composer.json'), 'utf-8')
        return composer.includes('"cakephp/cakephp"')
      } catch {
        try {
          return statSync(join(dir, 'config', 'app.php')).isFile()
        } catch {
          return false
        }
      }
    },
    supportedVersions: ['4.5', '5.0'],
    defaultVersion: '5.0',
    kind: 'Backend',

    async dev(dir, port) {
      const { execFile } = await import('child_process')
      await execFile('php', ['bin/cake', 'server', '-p', String(port || 8765)], { cwd: dir, stdio: 'inherit' })
    },

    async build(dir) {
      const { execFile } = await import('child_process')
      await execFile('composer', ['install', '--no-dev'], { cwd: dir, stdio: 'inherit' })
    },

    async test(dir) {
      const { execFile } = await import('child_process')
      await execFile('php', ['vendor/bin/phpunit'], { cwd: dir, stdio: 'inherit' })
    },

    async lint(dir) {
      const { execFile } = await import('child_process')
      await execFile('php', ['vendor/bin/phpcs', '--standard=CakePHP'], { cwd: dir, stdio: 'inherit' })
    },

    async format(dir) {
      const { execFile } = await import('child_process')
      await execFile('php', ['vendor/bin/phpcbf', '--standard=CakePHP'], { cwd: dir, stdio: 'inherit' })
    },

    async scaffold(name, options) {
      const dir = options?.dir || name
      const version = options?.version || '5.0'

      const dirs = [
        'config', 'src/Controller', 'src/Model', 'src/Model/Table', 'src/Model/Entity',
        'src/View', 'src/View/Ajax', 'templates', 'templates/Pages',
        'templates/layout', 'tests', 'tests/TestCase', 'tests/TestCase/Controller',
        'tests/TestCase/Model', 'tests/Fixture', 'webroot', 'webroot/css', 'webroot/js', 'webroot/img'
      ]
      for (const d of dirs) {
        mkdirSync(join(dir, d), { recursive: true })
      }

      const appName = name || 'app'
      const namespace = appName.replace(/[^a-zA-Z0-9_]/g, '_')

      writeFileSync(join(dir, 'composer.json'), JSON.stringify({
        name: `app/${appName}`,
        type: 'project',
        require: {
          php: '>=8.1',
          'cakephp/cakephp': version === '5.0' ? '^5.0' : '^4.5',
          'cakephp/migrations': '*',
          'cakephp/authentication': '^3.0'
        },
        'require-dev': {
          'cakephp/debug_kit': '*',
          'cakephp/bake': '*',
          'phpunit/phpunit': '^10.0'
        },
        autoload: {
          'psr-4': {
            `App\\`: 'src/'
          }
        },
        scripts: {
          test: 'php vendor/bin/phpunit'
        }
      }, null, 2) + '\n')

      writeFileSync(join(dir, 'config', 'app.php'), `<?php
return [
    'debug' => filter_var(env('DEBUG', true), FILTER_VALIDATE_BOOLEAN),
    'App' => [
        'namespace' => 'App',
        'encoding' => 'UTF-8',
        'defaultLocale' => env('APP_DEFAULT_LOCALE', 'en_US'),
        'base' => false,
        'dir' => 'src',
        'webroot' => 'webroot',
        'wwwRoot' => WWW_ROOT,
        'fullBaseUrl' => false,
        'imageUrl' => 'img/',
        'jsUrl' => 'js/',
        'cssUrl' => 'css/',
        'dateFormat' => 'Y-m-d',
        'timeFormat' => 'H:i:s',
        'datetimeFormat' => 'Y-m-d H:i:s',
    ],
    'Security' => [
        'salt' => env('SECURITY_SALT', '__SALT__'),
    ],
    'Datasources' => [
        'default' => [
            'host' => env('DB_HOST', 'localhost'),
            'username' => env('DB_USERNAME', 'root'),
            'password' => env('DB_PASSWORD', ''),
            'database' => env('DB_DATABASE', '${appName}'),
            'className' => 'Cake\\Database\\Connection',
            'driver' => 'Cake\\Database\\Driver\\Mysql',
            'encoding' => 'utf8mb4',
            'timezone' => 'UTC',
            'cacheMetadata' => true,
        ],
    ],
];
`)

      writeFileSync(join(dir, 'config', 'routes.php'), `<?php
use Cake\\Routing\\Route\\DashedRoute;
use Cake\\Routing\\RouteBuilder;

return function (RouteBuilder $routes) {
    $routes->setRouteClass(DashedRoute::class);

    $routes->scope('/', function (RouteBuilder $builder) {
        $builder->connect('/', 'Pages::display', ['home']);
        $builder->connect('/pages/*', 'Pages::display');
        $builder->fallback();
    });
};
`)

      writeFileSync(join(dir, 'src', 'Controller', 'AppController.php'), `<?php
namespace App\\Controller;

use Cake\\Controller\\Controller;

class AppController extends Controller
{
    public function initialize(): void
    {
        parent::initialize();
        $this->loadComponent('Flash');
    }
}
`)

      writeFileSync(join(dir, 'src', 'Controller', 'PagesController.php'), `<?php
namespace App\\Controller;

use Cake\\Core\\Configure;

class PagesController extends AppController
{
    public function display(string ...$path): void
    {
        $page = $path ? implode('/', $path) : 'home';
        $this->set(compact('page'));
    }
}
`)

      writeFileSync(join(dir, 'src', 'Model', 'Entity', 'User.php'), `<?php
namespace App\\Model\\Entity;

use Cake\\ORM\\Entity;

class User extends Entity
{
    protected array $_accessible = [
        'username' => true,
        'email' => true,
        'password' => true,
        'role' => true,
        '*' => false,
    ];

    protected array $_hidden = ['password'];
}
`)

      writeFileSync(join(dir, 'src', 'Model', 'Table', 'UsersTable.php'), `<?php
namespace App\\Model\\Table;

use Cake\\ORM\\Table;
use Cake\\Validation\\Validator;

class UsersTable extends Table
{
    public function initialize(array $config): void
    {
        parent::initialize($config);
        $this->setTable('users');
        $this->setDisplayField('username');
        $this->setPrimaryKey('id');
        $this->addBehavior('Timestamp');
    }

    public function validationDefault(Validator $validator): Validator
    {
        $validator
            ->notEmptyString('username')
            ->maxLength('username', 255)
            ->notEmptyString('email')
            ->email('email')
            ->notEmptyString('password')
            ->minLength('password', 6);
        return $validator;
    }
}
`)

      writeFileSync(join(dir, 'templates', 'layout', 'default.php'), `<!DOCTYPE html>
<html>
<head>
    <?= \$this->Html->charset() ?>
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title><?= \$this->fetch('title') ?></title>
    <?= \$this->Html->meta('icon') ?>
    <?= \$this->Html->css(['normalize.min', 'milligram.min', 'fonts', 'cake']) ?>
    <?= \$this->fetch('meta') ?>
    <?= \$this->fetch('css') ?>
</head>
<body>
    <main>
        <div class="container">
            <?= \$this->Flash->render() ?>
            <?= \$this->fetch('content') ?>
        </div>
    </main>
    <?= \$this->fetch('script') ?>
</body>
</html>
`)

      writeFileSync(join(dir, 'templates', 'Pages', 'home.php'), `<div class="page index">
    <h1><?= __('Welcome to ${appName}') ?></h1>
    <p><?= __('CakePHP application scaffolded successfully.') ?></p>
</div>
`)

      writeFileSync(join(dir, 'webroot', 'index.php'), `<?php
require dirname(__DIR__) . '/vendor/autoload.php';
use Cake\\Core\\Configure;
use Cake\\Http\\Server;

\$server = new Server();
\$server->emit(\$server->run());
`)

      writeFileSync(join(dir, 'webroot', 'css', 'app.css'), `/* ${appName} styles */\n`)

      writeFileSync(join(dir, 'tests', 'bootstrap.php'), `<?php
require dirname(__DIR__) . '/vendor/autoload.php';
`)

      writeFileSync(join(dir, '.gitignore'), `/vendor/
/composer.lock
/config/app_local.php
/tmp/
/logs/
.env
`)
    }
  }
}

function existsSync(p) { try { return statSync(p).isFile() } catch { return false } }
