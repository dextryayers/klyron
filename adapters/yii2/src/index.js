import { readFileSync, statSync, writeFileSync, mkdirSync } from 'fs'
import { join } from 'path'

export function createAdapter() {
  return {
    name: 'Yii2',
    detect(dir) {
      try {
        const composer = readFileSync(join(dir, 'composer.json'), 'utf-8')
        return composer.includes('"yiisoft/yii2"')
      } catch {
        try {
          return statSync(join(dir, 'config', 'web.php')).isFile()
        } catch {
          return false
        }
      }
    },
    supportedVersions: ['2.0'],
    defaultVersion: '2.0',
    kind: 'Backend',

    async dev(dir, port) {
      const { execFile } = await import('child_process')
      await execFile('php', ['yii', 'serve', `--port=${port || 8080}`], { cwd: dir, stdio: 'inherit' })
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
      await execFile('php', ['vendor/bin/phpcs', '--standard=Yii2'], { cwd: dir, stdio: 'inherit' })
    },

    async format(dir) {
      const { execFile } = await import('child_process')
      await execFile('php', ['vendor/bin/phpcbf', '--standard=Yii2'], { cwd: dir, stdio: 'inherit' })
    },

    async scaffold(name, options) {
      const dir = options?.dir || name
      const appName = name || 'app'

      const dirs = [
        'config', 'controllers', 'models', 'views', 'views/layouts', 'views/site',
        'mail', 'web', 'web/css', 'web/js', 'web/assets',
        'tests', 'tests/unit', 'tests/functional', 'tests/acceptance',
        'runtime', 'vendor'
      ]
      for (const d of dirs) {
        mkdirSync(join(dir, d), { recursive: true })
      }

      writeFileSync(join(dir, 'composer.json'), JSON.stringify({
        name: `app/${appName}`,
        type: 'project',
        require: {
          php: '>=8.0',
          'yiisoft/yii2': '^2.0',
          'yiisoft/yii2-bootstrap5': '*',
          'yiisoft/yii2-swiftmailer': '*'
        },
        'require-dev': {
          'yiisoft/yii2-debug': '*',
          'yiisoft/yii2-gii': '*',
          'phpunit/phpunit': '^9.0'
        },
        autoload: {
          'psr-4': {
            'app\\': ''
          }
        },
        scripts: {
          test: 'php vendor/bin/phpunit'
        }
      }, null, 2) + '\n')

      writeFileSync(join(dir, 'config', 'web.php'), `<?php
\$params = require __DIR__ . '/params.php';

return [
    'id' => '${appName}',
    'basePath' => dirname(__DIR__),
    'bootstrap' => ['log'],
    'aliases' => [
        '@webroot' => dirname(__DIR__) . '/web',
    ],
    'components' => [
        'request' => [
            'cookieValidationKey' => '__COOKIE_KEY__',
        ],
        'cache' => [
            'class' => 'yii\\caching\\FileCache',
        ],
        'user' => [
            'identityClass' => 'app\\models\\User',
            'enableAutoLogin' => true,
        ],
        'errorHandler' => [
            'errorAction' => 'site/error',
        ],
        'log' => [
            'traceLevel' => YII_DEBUG ? 3 : 0,
            'targets' => [
                [
                    'class' => 'yii\\log\\FileTarget',
                    'levels' => ['error', 'warning'],
                ],
            ],
        ],
        'db' => require __DIR__ . '/db.php',
        'urlManager' => [
            'enablePrettyUrl' => true,
            'showScriptName' => false,
        ],
    ],
    'params' => \$params,
];
`)

      writeFileSync(join(dir, 'config', 'console.php'), `<?php
\$params = require __DIR__ . '/params.php';

return [
    'id' => '${appName}-console',
    'basePath' => dirname(__DIR__),
    'bootstrap' => ['log'],
    'controllerNamespace' => 'app\\commands',
    'components' => [
        'cache' => [
            'class' => 'yii\\caching\\FileCache',
        ],
        'log' => [
            'targets' => [
                [
                    'class' => 'yii\\log\\FileTarget',
                    'levels' => ['error', 'warning'],
                ],
            ],
        ],
        'db' => require __DIR__ . '/db.php',
    ],
    'params' => \$params,
];
`)

      writeFileSync(join(dir, 'config', 'params.php'), `<?php
return [
    'adminEmail' => 'admin@${appName}.com',
    'senderEmail' => 'noreply@${appName}.com',
    'senderName' => '${appName} mailer',
];
`)

      writeFileSync(join(dir, 'config', 'db.php'), `<?php
return [
    'class' => 'yii\\db\\Connection',
    'dsn' => getenv('DB_DSN') ?: 'mysql:host=localhost;dbname=${appName}',
    'username' => getenv('DB_USERNAME') ?: 'root',
    'password' => getenv('DB_PASSWORD') ?: '',
    'charset' => 'utf8',
];
`)

      writeFileSync(join(dir, 'controllers', 'SiteController.php'), `<?php
namespace app\\controllers;

use yii\\web\\Controller;

class SiteController extends Controller
{
    public function actions()
    {
        return [
            'error' => [
                'class' => 'yii\\web\\ErrorAction',
            ],
        ];
    }

    public function actionIndex()
    {
        return \$this->render('index');
    }

    public function actionAbout()
    {
        return \$this->render('about');
    }
}
`)

      writeFileSync(join(dir, 'models', 'User.php'), `<?php
namespace app\\models;

use yii\\db\\ActiveRecord;
use yii\\web\\IdentityInterface;

class User extends ActiveRecord implements IdentityInterface
{
    public static function findIdentity(\$id)
    {
        return static::findOne(\$id);
    }

    public static function findIdentityByAccessToken(\$token, \$type = null)
    {
        return static::findOne(['accessToken' => \$token]);
    }

    public function getId()
    {
        return \$this->id;
    }

    public function getAuthKey()
    {
        return \$this->authKey;
    }

    public function validateAuthKey(\$authKey)
    {
        return \$this->authKey === \$authKey;
    }
}
`)

      writeFileSync(join(dir, 'views', 'layouts', 'main.php'), `<?php
use yii\\helpers\\Html;
?>
<?php \$this->beginPage() ?>
<!DOCTYPE html>
<html>
<head>
    <meta charset="<?= Yii::\$app->charset ?>">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <?= Html::csrfMetaTags() ?>
    <title><?= Html::encode(\$this->title) ?></title>
    <?php \$this->head() ?>
</head>
<body>
<?php \$this->beginBody() ?>
    <div class="wrap">
        <?= \$content ?>
    </div>
<?php \$this->endBody() ?>
</body>
</html>
<?php \$this->endPage() ?>
`)

      writeFileSync(join(dir, 'views', 'site', 'index.php'), `<?php
\$this->title = '${appName}';
?>
<div class="site-index">
    <h1><?= Yii::\$app->name ?></h1>
    <p><?= Yii::\$t('app', 'Welcome to {appName}!', ['appName' => '${appName}']) ?></p>
</div>
`)

      writeFileSync(join(dir, 'views', 'site', 'about.php'), `<?php
\$this->title = 'About';
?>
<div class="site-about">
    <h1><?= \$this->title ?></h1>
    <p><?= Yii::\$t('app', 'About {appName}', ['appName' => '${appName}']) ?></p>
</div>
`)

      writeFileSync(join(dir, 'web', 'index.php'), `<?php
defined('YII_DEBUG') or define('YII_DEBUG', true);
defined('YII_ENV') or define('YII_ENV', 'dev');

require __DIR__ . '/../vendor/autoload.php';
require __DIR__ . '/../vendor/yiisoft/yii2/Yii.php';

\$config = require __DIR__ . '/../config/web.php';
(new yii\\web\\Application(\$config))->run();
`)

      writeFileSync(join(dir, 'web', 'css', 'site.css'), `/* ${appName} styles */\n`)

      writeFileSync(join(dir, 'mail', 'layouts', 'html.php'), `<?php
use yii\\helpers\\Html;
?>
<!DOCTYPE html>
<html>
<head><title><?= Html::encode(\$this->title) ?></title></head>
<body><?= \$content ?></body>
</html>
`)

      writeFileSync(join(dir, 'tests', 'bootstrap.php'), `<?php
require dirname(__DIR__) . '/vendor/autoload.php';
`)

      writeFileSync(join(dir, '.gitignore'), `/vendor/
/composer.lock
/runtime/
.env
`)
    }
  }
}

function existsSync(p) { try { return statSync(p).isFile() } catch { return false } }
