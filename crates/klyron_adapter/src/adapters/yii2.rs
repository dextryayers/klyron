use std::path::Path;
use async_trait::async_trait;
use anyhow::Result;
use crate::{FrameworkAdapter, BuildOptions, ScaffoldOptions, FrameworkKind};

pub struct Yii2Adapter;

fn version_deps(version: &str) -> (&'static str, &'static str) {
    match version {
        "2.0" => ("^2.0", "^7.4|^8.0"),
        _ => ("^2.0", "^7.4|^8.0"),
    }
}

#[async_trait]
impl FrameworkAdapter for Yii2Adapter {
    fn name(&self) -> &'static str { "yii2" }

    fn detect(&self, dir: &Path) -> bool {
        let composer = dir.join("composer.json");
        if let Ok(content) = std::fs::read_to_string(composer) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(require) = json.get("require") {
                    for key in &["yiisoft/yii2", "yiisoft/yii2-web", "yiisoft/yii2-base"] {
                        if require.get(*key).is_some() { return true; }
                    }
                }
            }
        }
        dir.join("config/web.php").exists() && dir.join("yii").exists()
    }

    fn supported_versions(&self) -> Vec<&'static str> { vec!["2.0"] }
    fn default_version(&self) -> &'static str { "2.0" }
    fn kind(&self) -> FrameworkKind { FrameworkKind::Backend }

    async fn dev(&self, dir: &Path, port: Option<u16>) -> Result<()> {
        let mut cmd = tokio::process::Command::new("php");
        cmd.args(["yii", "serve"]).current_dir(dir);
        if let Some(p) = port { cmd.args([&format!("0.0.0.0:{}", p)]); }
        cmd.status().await?; Ok(())
    }

    async fn build(&self, _dir: &Path, _opts: BuildOptions) -> Result<()> { Ok(()) }

    async fn test(&self, dir: &Path, _filter: Option<&str>) -> Result<()> {
        tokio::process::Command::new("php")
            .args(["./vendor/bin/phpunit"])
            .current_dir(dir).status().await?;
        Ok(())
    }

    async fn lint(&self, dir: &Path, _fix: bool) -> Result<()> {
        tokio::process::Command::new("php")
            .args(["./vendor/bin/phpcs", "--standard=Yii2", "--extensions=php", "src/"])
            .current_dir(dir).status().await?;
        Ok(())
    }

    async fn format(&self, dir: &Path, write: bool) -> Result<()> {
        if write {
            tokio::process::Command::new("php")
                .args(["./vendor/bin/phpcbf", "--standard=Yii2", "--extensions=php", "src/"])
                .current_dir(dir).status().await?;
        } else {
            tokio::process::Command::new("php")
                .args(["./vendor/bin/phpcs", "--standard=Yii2", "--extensions=php", "src/"])
                .current_dir(dir).status().await?;
        }
        Ok(())
    }

    fn external_scaffold_command(&self, name: &str, _version: Option<&str>) -> Option<(String, Vec<String>)> {
        Some(("composer".into(), vec![
            "create-project".into(),
            "yiisoft/yii2-app-basic".into(),
            name.into(),
        ]))
    }

    async fn scaffold(&self, name: &str, options: ScaffoldOptions) -> Result<()> {
        if options.external {
            if let Some((cmd, args)) = self.external_scaffold_command(name, options.version.as_deref()) {
                let status = std::process::Command::new(&cmd).args(&args).current_dir(&options.dir).status()?;
                if !status.success() { anyhow::bail!("External scaffolding failed"); }
                return Ok(());
            }
        }
        let version = options.version.as_deref().unwrap_or("2.0");
        let (_fw_dep, php_req) = version_deps(version);
        let vars = &options.template_vars;
        let project_dir = options.dir.join(name);

        for d in &[
            "config",
            "controllers", "models", "views", "views/layouts",
            "views/site", "mail", "widgets",
            "web", "web/assets", "web/css", "web/js",
            "runtime", "tests",
        ] {
            std::fs::create_dir_all(project_dir.join(d))?;
        }

        std::fs::write(project_dir.join("composer.json"),
            klyron_template::TemplateEngine::render_static(
                &format!(r#"{{
  "name": "app/{{ name }}",
  "type": "project",
  "description": "Yii2 project",
  "require": {{
    "php": "{}",
    "yiisoft/yii2": "{}",
    "yiisoft/yii2-bootstrap5": "^2.0",
    "yiisoft/yii2-symfonymailer": "^2.0"
  }},
  "require-dev": {{
    "yiisoft/yii2-debug": "^2.1",
    "yiisoft/yii2-gii": "^2.2",
    "phpunit/phpunit": "^9.6"
  }},
  "autoload": {{
    "psr-4": {{ "app\\": "", "app\\controllers\\": "controllers/", "app\\models\\": "models/", "app\\widgets\\": "widgets/" }}
  }},
  "scripts": {{
    "post-create-project-cmd": [
      "php yii init --ansi"
    ]
  }}
}}"#, php_req, version),
            vars))?;

        std::fs::write(project_dir.join(".env"),
            r#"YII_DEBUG=true
YII_ENV=dev
DB_CONNECTION=mysql
DB_HOST=127.0.0.1
DB_PORT=3306
DB_DATABASE=yii2
DB_USERNAME=root
DB_PASSWORD=
"#)?;

        std::fs::write(project_dir.join(".gitignore"),
            "/vendor\n/.env\n/runtime/*\n/web/assets/*\n!web/assets/.gitkeep\n.DS_Store\n")?;

        std::fs::write(project_dir.join("yii"),
            r#"#!/usr/bin/env php
<?php
defined('YII_DEBUG') or define('YII_DEBUG', true);
defined('YII_ENV') or define('YII_ENV', 'dev');
require __DIR__ . '/vendor/autoload.php';
require __DIR__ . '/vendor/yiisoft/yii2/Yii.php';
$config = require __DIR__ . '/config/console.php';
(new yii\console\Application($config))->run();
"#)?;

        std::fs::write(project_dir.join("web/index.php"),
            r#"<?php
defined('YII_DEBUG') or define('YII_DEBUG', true);
defined('YII_ENV') or define('YII_ENV', 'dev');
require __DIR__ . '/../vendor/autoload.php';
require __DIR__ . '/../vendor/yiisoft/yii2/Yii.php';
$config = require __DIR__ . '/../config/web.php';
(new yii\web\Application($config))->run();
"#)?;

        std::fs::write(project_dir.join("config/web.php"),
            r#"<?php
$params = require __DIR__ . '/params.php';
$db = require __DIR__ . '/db.php';
return [
    'id' => 'basic',
    'basePath' => dirname(__DIR__),
    'bootstrap' => ['log'],
    'aliases' => ['@web' => '/'],
    'components' => [
        'request' => ['cookieValidationKey' => 'change-this-key'],
        'cache' => ['class' => 'yii\caching\FileCache'],
        'user' => [
            'identityClass' => 'app\models\User',
            'enableAutoLogin' => true,
        ],
        'errorHandler' => ['errorAction' => 'site/error'],
        'mailer' => [
            'class' => 'yii\symfonymailer\Mailer',
            'useFileTransport' => true,
        ],
        'log' => ['traceLevel' => YII_DEBUG ? 3 : 0, 'targets' => [['class' => 'yii\log\FileTarget', 'levels' => ['error', 'warning']]]],
        'db' => $db,
        'urlManager' => ['enablePrettyUrl' => true, 'showScriptName' => false],
    ],
    'params' => $params,
];
"#)?;

        std::fs::write(project_dir.join("config/console.php"),
            r#"<?php
$params = require __DIR__ . '/params.php';
$db = require __DIR__ . '/db.php';
return [
    'id' => 'basic-console',
    'basePath' => dirname(__DIR__),
    'bootstrap' => ['log', 'gii'],
    'controllerNamespace' => 'app\commands',
    'aliases' => ['@web' => '/'],
    'components' => [
        'cache' => ['class' => 'yii\caching\FileCache'],
        'log' => ['traceLevel' => YII_DEBUG ? 3 : 0, 'targets' => [['class' => 'yii\log\FileTarget', 'levels' => ['error', 'warning']]]],
        'db' => $db,
    ],
    'params' => $params,
    'controllerMap' => ['migrate' => ['class' => 'yii\console\controllers\MigrateController', 'migrationPath' => null, 'migrationNamespaces' => ['app\migrations']]],
];
"#)?;

        std::fs::write(project_dir.join("config/params.php"),
            r#"<?php return ['adminEmail' => 'admin@example.com', 'senderEmail' => 'noreply@example.com', 'senderName' => 'Example.com mailer'];"#)?;

        std::fs::write(project_dir.join("config/db.php"),
            r#"<?php return ['class' => 'yii\db\Connection', 'dsn' => env('DB_DSN', 'mysql:host=127.0.0.1;dbname=yii2'), 'username' => env('DB_USERNAME', 'root'), 'password' => env('DB_PASSWORD', ''), 'charset' => 'utf8'];"#)?;

        std::fs::write(project_dir.join("controllers/SiteController.php"),
            r#"<?php
namespace app\controllers;
use Yii;
use yii\web\Controller;
class SiteController extends Controller
{
    public function actions(): array
    {
        return ['error' => ['class' => 'yii\web\ErrorAction']];
    }
    public function actionIndex(): string
    {
        return $this->render('index');
    }
}
"#)?;

        std::fs::write(project_dir.join("models/User.php"),
            r#"<?php
namespace app\models;
use yii\db\ActiveRecord;
use yii\web\IdentityInterface;
class User extends ActiveRecord implements IdentityInterface
{
    public static function findIdentity($id): static
    {
        return static::findOne($id);
    }
    public static function findIdentityByAccessToken($token, $type = null): ?static
    {
        return null;
    }
    public function getId(): int
    {
        return $this->id;
    }
    public function getAuthKey(): ?string
    {
        return null;
    }
    public function validateAuthKey($authKey): bool
    {
        return false;
    }
}
"#)?;

        std::fs::write(project_dir.join("models/LoginForm.php"),
            r#"<?php
namespace app\models;
use Yii;
use yii\base\Model;
class LoginForm extends Model
{
    public string $username = '';
    public string $password = '';
    private ?User $_user = null;
    public function rules(): array
    {
        return [['username', 'password'], 'required'], [['password'], 'validatePassword']];
    }
    public function validatePassword(string $attribute, ?array $params): void
    {
        if (!$this->hasErrors()) {
            $user = $this->getUser();
            if (!$user || !$user->validatePassword($this->password)) {
                $this->addError($attribute, 'Incorrect username or password.');
            }
        }
    }
    public function login(): bool
    {
        if ($this->validate()) {
            return Yii::$app->user->login($this->getUser());
        }
        return false;
    }
    public function getUser(): ?User
    {
        if ($this->_user === null) {
            $this->_user = User::findByUsername($this->username);
        }
        return $this->_user;
    }
}
"#)?;

        std::fs::write(project_dir.join("views/layouts/main.php"),
            klyron_template::TemplateEngine::render_static(
                r#"<?php use yii\helpers\Html; ?>
<!DOCTYPE html>
<html>
<head><meta charset="<?= Yii::$app->charset ?>"><title><?= Html::encode($this->title) ?> | {{ name }}</title></head>
<body>
    <div class="wrap">
        <header><h1>{{ name }}</h1></header>
        <main><?= $content ?></main>
    </div>
</body>
</html>"#, vars))?;

        std::fs::write(project_dir.join("views/site/index.php"),
            klyron_template::TemplateEngine::render_static(
                r#"<?php $this->title = 'Home'; ?>
<h1>Welcome to {{ name }}</h1>
<p>Your Yii2 application is ready.</p>"#, vars))?;

        std::fs::write(project_dir.join("views/site/error.php"),
            r#"<?php $this->title = 'Error'; ?>
<h1><?= nl2br(Html::encode($exception->getMessage())) ?></h1>"#)?;

        std::fs::write(project_dir.join("mail/layouts/html.php"),
            r#"<?php use yii\helpers\Html; ?>
<!DOCTYPE html><html><body><?= $content ?></body></html>"#)?;

        std::fs::write(project_dir.join("web/css/site.css"),
            r#"body { font-family: 'Helvetica Neue', Helvetica, Arial, sans-serif; margin: 0; padding: 0; }"#)?;

        std::fs::write(project_dir.join("phpunit.xml.dist"),
            r#"<?xml version="1.0" encoding="UTF-8"?>
<phpunit xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
         xsi:noNamespaceSchemaLocation="vendor/phpunit/phpunit/phpunit.xsd"
         bootstrap="tests/bootstrap.php" colors="true">
    <testsuites><testsuite name="App"><directory>tests</directory></testsuite></testsuites>
</phpunit>
"#)?;

        std::fs::write(project_dir.join("tests/bootstrap.php"),
            r#"<?php
require dirname(__DIR__) . '/vendor/autoload.php';
"#)?;

        std::fs::write(project_dir.join("runtime/.gitkeep"), "")?;
        std::fs::write(project_dir.join("web/assets/.gitkeep"), "")?;

        println!("Yii2 {} app created: {}", version, project_dir.display());
        Ok(())
    }
}
