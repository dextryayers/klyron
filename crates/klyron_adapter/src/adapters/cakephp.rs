use std::path::Path;
use async_trait::async_trait;
use anyhow::Result;
use crate::{FrameworkAdapter, BuildOptions, ScaffoldOptions, FrameworkKind};

pub struct CakePHPAdapter;

fn version_deps(version: &str) -> (&'static str, &'static str) {
    match version {
        "4.4" => ("^4.4", "^7.4|^8.0"),
        "4.5" => ("^4.5", "^8.1"),
        "5.0" => ("^5.0", "^8.1"),
        _ => ("^4.5", "^8.1"),
    }
}

#[async_trait]
impl FrameworkAdapter for CakePHPAdapter {
    fn name(&self) -> &'static str { "cakephp" }

    fn detect(&self, dir: &Path) -> bool {
        let composer = dir.join("composer.json");
        if let Ok(content) = std::fs::read_to_string(composer) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(require) = json.get("require") {
                    for key in &["cakephp/cakephp"] {
                        if require.get(*key).is_some() { return true; }
                    }
                }
            }
        }
        dir.join("config/app.php").exists() && dir.join("bin/cake").exists()
    }

    fn supported_versions(&self) -> Vec<&'static str> { vec!["4.4", "4.5", "5.0"] }
    fn default_version(&self) -> &'static str { "4.5" }
    fn kind(&self) -> FrameworkKind { FrameworkKind::Backend }

    async fn dev(&self, dir: &Path, port: Option<u16>) -> Result<()> {
        let mut cmd = tokio::process::Command::new("php");
        cmd.args(["bin/cake.php", "server"]).current_dir(dir);
        if let Some(p) = port { cmd.args(["-p", &p.to_string()]); }
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
            .args(["./vendor/bin/phpcs", "--standard=CakePHP", "--extensions=php", "src/", "tests/"])
            .current_dir(dir).status().await?;
        Ok(())
    }

    async fn format(&self, dir: &Path, write: bool) -> Result<()> {
        if write {
            tokio::process::Command::new("php")
                .args(["./vendor/bin/phpcbf", "--standard=CakePHP", "--extensions=php", "src/", "tests/"])
                .current_dir(dir).status().await?;
        } else {
            tokio::process::Command::new("php")
                .args(["./vendor/bin/phpcs", "--standard=CakePHP", "--extensions=php", "src/", "tests/"])
                .current_dir(dir).status().await?;
        }
        Ok(())
    }

    fn external_scaffold_command(&self, name: &str, _version: Option<&str>) -> Option<(String, Vec<String>)> {
        Some(("composer".into(), vec![
            "create-project".into(),
            "cakephp/app".into(),
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
        let version = options.version.as_deref().unwrap_or("4.5");
        let (_fw_dep, php_req) = version_deps(version);
        let vars = &options.template_vars;
        let project_dir = options.dir.join(name);

        for d in &[
            "config",
            "src/Controller", "src/Model/Entity", "src/Model/Table",
            "src/View", "src/View/Helper", "src/View/Cell",
            "src/Shell", "src/Mailer", "src/Command",
            "templates", "templates/layout", "templates/pages",
            "templates/element", "templates/email/html", "templates/email/text",
            "webroot", "webroot/css", "webroot/js", "webroot/img",
            "logs", "tmp", "tests/Fixture", "tests/TestCase",
        ] {
            std::fs::create_dir_all(project_dir.join(d))?;
        }

        std::fs::write(project_dir.join("composer.json"),
            klyron_template::TemplateEngine::render_static(
                &format!(r#"{{
  "name": "app/{{ name }}",
  "type": "project",
  "description": "CakePHP project",
  "require": {{
    "php": "{}",
    "cakephp/cakephp": "{}",
    "cakephp/authentication": "^3.0",
    "cakephp/authorization": "^3.0"
  }},
  "require-dev": {{
    "cakephp/bake": "^3.0",
    "cakephp/debug_kit": "^5.0",
    "cakephp/cakephp-codesniffer": "^5.0",
    "phpunit/phpunit": "^10.0"
  }},
  "autoload": {{
    "psr-4": {{ "App\\": "src/" }}
  }},
  "autoload-dev": {{
    "psr-4": {{ "App\\Test\\": "tests/", "Cake\\Test\\": "vendor/cakephp/cakephp/tests/" }}
  }},
  "scripts": {{
    "post-create-project-cmd": [
      "php bin/cake.php bootstrap"
    ]
  }}
}}"#, php_req, version),
            vars))?;

        std::fs::write(project_dir.join(".env"),
            r#"APP_NAME=CakePHP
DEBUG=true
APP_ENCODING=UTF-8
APP_DEFAULT_LOCALE=en_US
APP_DEFAULT_TIMEZONE=UTC
SECURITY_SALT=change-this-salt-key
DATABASE_URL=mysql://root:@localhost/cakephp
"#)?;

        std::fs::write(project_dir.join(".gitignore"),
            "/vendor\n/.env\n/tmp/*\n/logs/*\n!logs/.gitkeep\n!tmp/.gitkeep\n.DS_Store\n")?;

        std::fs::write(project_dir.join("webroot/index.php"),
            r#"<?php
require dirname(__DIR__) . '/vendor/autoload.php';
use Cake\Http\Server;
$server = new Server(dirname(__DIR__) . '/config/app.php');
$server->emit($server->run());
"#)?;

        std::fs::write(project_dir.join("config/app.php"),
            r#"<?php
return [
    'name' => env('APP_NAME', 'CakePHP'),
    'debug' => filter_var(env('DEBUG', true), FILTER_VALIDATE_BOOLEAN),
    'Security' => ['salt' => env('SECURITY_SALT', 'change-this-salt-key')],
    'Datasources' => [
        'default' => [
            'className' => 'Cake\Database\Connection',
            'driver' => 'Cake\Database\Driver\Mysql',
            'url' => env('DATABASE_URL'),
        ],
    ],
];
"#)?;

        std::fs::write(project_dir.join("config/routes.php"),
            r#"<?php
use Cake\Routing\Route\DashedRoute;
use Cake\Routing\RouteBuilder;
return function (RouteBuilder $routes) {
    $routes->setRouteClass(DashedRoute::class);
    $routes->scope('/', function (RouteBuilder $builder) {
        $builder->connect('/', ['controller' => 'Pages', 'action' => 'display', 'home']);
        $builder->fallbacks();
    });
};
"#)?;

        std::fs::write(project_dir.join("config/bootstrap.php"),
            r#"<?php
require dirname(__DIR__) . '/vendor/autoload.php';
use Cake\Core\Configure;
use Cake\Datasource\ConnectionManager;
try {
    Configure::load('app', 'default');
} catch (\Exception $e) {
    exit('Unable to load config/app.php. Create it by copying config/app.php.default to config/app.php.');
}
"#)?;

        std::fs::write(project_dir.join("src/Controller/AppController.php"),
            r#"<?php
namespace App\Controller;
use Cake\Controller\Controller;
class AppController extends Controller
{
    public function initialize(): void
    {
        parent::initialize();
        $this->loadComponent('Authentication.Authentication');
    }
}
"#)?;

        std::fs::write(project_dir.join("src/Controller/PagesController.php"),
            r#"<?php
namespace App\Controller;
use Cake\Core\Configure;
use Cake\Http\Exception\ForbiddenException;
use Cake\Http\Exception\NotFoundException;
use Cake\View\Exception\MissingTemplateException;
class PagesController extends AppController
{
    public function display(string ...$path): void
    {
        if (!$path) {
            $path = ['home'];
        }
        $this->set('page', $path[0]);
        $this->render('/pages/' . $path[0]);
    }
}
"#)?;

        std::fs::write(project_dir.join("src/Model/Entity/User.php"),
            r#"<?php
namespace App\Model\Entity;
use Cake\ORM\Entity;
class User extends Entity
{
    protected array $_accessible = [
        'email' => true,
        'password' => true,
        'name' => true,
        '*' => false,
    ];
    protected array $_hidden = ['password'];
}
"#)?;

        std::fs::write(project_dir.join("src/Model/Table/UsersTable.php"),
            r#"<?php
namespace App\Model\Table;
use Cake\ORM\Table;
use Cake\Validation\Validator;
class UsersTable extends Table
{
    public function initialize(array $config): void
    {
        parent::initialize($config);
        $this->setTable('users');
        $this->setDisplayField('email');
        $this->setPrimaryKey('id');
        $this->addBehavior('Timestamp');
    }
    public function validationDefault(Validator $validator): Validator
    {
        $validator
            ->email('email')
            ->notEmptyString('email')
            ->requirePresence('email', 'create');
        return $validator;
    }
}
"#)?;

        std::fs::write(project_dir.join("templates/layout/default.php"),
            klyron_template::TemplateEngine::render_static(
                r#"<!DOCTYPE html>
<html>
<head><meta charset="UTF-8"><title><?= h($this->fetch('title')) ?> | {{ name }}</title></head>
<body>
    <header><h1>{{ name }}</h1></header>
    <main><?= $this->fetch('content') ?></main>
</body>
</html>"#, vars))?;

        std::fs::write(project_dir.join("templates/pages/home.php"),
            klyron_template::TemplateEngine::render_static(
                r#"<?php $this->assign('title', 'Home'); ?>
<h1>Welcome to {{ name }}</h1>
<p>Your CakePHP application is ready.</p>"#, vars))?;

        std::fs::write(project_dir.join("templates/email/html/default.php"),
            r#"<p>Email content</p>"#)?;

        std::fs::write(project_dir.join("templates/email/text/default.php"),
            r#"Email content"#)?;

        std::fs::write(project_dir.join("webroot/css/style.css"),
            r#"body { font-family: 'Helvetica Neue', Helvetica, Arial, sans-serif; margin: 0; padding: 0; }"#)?;

        std::fs::write(project_dir.join("phpunit.xml.dist"),
            r#"<?xml version="1.0" encoding="UTF-8"?>
<phpunit xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
         xsi:noNamespaceSchemaLocation="vendor/phpunit/phpunit/phpunit.xsd"
         bootstrap="vendor/autoload.php" colors="true">
    <testsuites><testsuite name="App"><directory>tests/TestCase</directory></testsuite></testsuites>
    <php><env name="APP_ENV" value="test"/></php>
</phpunit>
"#)?;

        std::fs::write(project_dir.join("logs/.gitkeep"), "")?;
        std::fs::write(project_dir.join("tmp/.gitkeep"), "")?;

        println!("CakePHP {} app created: {}", version, project_dir.display());
        Ok(())
    }
}
