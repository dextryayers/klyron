use std::path::Path;
use async_trait::async_trait;
use anyhow::Result;
use crate::{FrameworkAdapter, BuildOptions, ScaffoldOptions, FrameworkKind};

pub struct CodeIgniterAdapter;

fn version_php(version: &str) -> (&'static str, &'static str) {
    match version {
        "4.4" => ("^4.4", "^7.4|^8.0"),
        "4.5" => ("^4.5", "^8.1"),
        _ => ("^4.5", "^8.1"),
    }
}

#[allow(dead_code)]
fn detect_ci_version(dir: &Path) -> Option<&'static str> {
    let env_path = dir.join("spark");
    if env_path.exists() {
        if let Ok(content) = std::fs::read_to_string(dir.join("composer.json")) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(req) = json.get("require") {
                    if let Some(ci) = req.get("codeigniter4/framework").and_then(|v| v.as_str()) {
                        if ci.contains("4.4") || ci.contains("^4.4") || ci.contains("~4.4") { return Some("4.4"); }
                        if ci.contains("4.5") || ci.contains("^4.5") { return Some("4.5"); }
                    }
                }
            }
        }
        return Some("4.5");
    }
    None
}

#[async_trait]
impl FrameworkAdapter for CodeIgniterAdapter {
    fn name(&self) -> &'static str { "codeigniter" }

    fn detect(&self, dir: &Path) -> bool {
        dir.join("spark").exists() || dir.join("app/Config").exists() || dir.join("system").exists()
    }

    fn supported_versions(&self) -> Vec<&'static str> { vec!["4.4", "4.5"] }
    fn default_version(&self) -> &'static str { "4.5" }
    fn kind(&self) -> FrameworkKind { FrameworkKind::Backend }

    async fn dev(&self, dir: &Path, port: Option<u16>) -> Result<()> {
        let mut cmd = tokio::process::Command::new("php");
        cmd.args(["spark", "serve"]).current_dir(dir);
        if let Some(p) = port { cmd.arg(format!("--port={}", p)); }
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
            .args(["./vendor/bin/php-cs-fixer", "fix", "--dry-run", "--diff"])
            .current_dir(dir).status().await?;
        Ok(())
    }

    async fn format(&self, dir: &Path, write: bool) -> Result<()> {
        if write {
            tokio::process::Command::new("php")
                .args(["./vendor/bin/php-cs-fixer", "fix"])
                .current_dir(dir).status().await?;
        } else {
            tokio::process::Command::new("php")
                .args(["./vendor/bin/php-cs-fixer", "fix", "--dry-run", "--diff"])
                .current_dir(dir).status().await?;
        }
        Ok(())
    }

    fn external_scaffold_command(&self, name: &str, _version: Option<&str>) -> Option<(String, Vec<String>)> {
        Some(("composer".into(), vec![
            "create-project".into(),
            "codeigniter4/appstarter".into(),
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
        let (_ci_ver, php_req) = version_php(version);
        let vars = &options.template_vars;
        let project_dir = options.dir.join(name);

        for d in &[
            "app/Config", "app/Controllers", "app/Database/Migrations",
            "app/Database/Seeds", "app/Models", "app/Views",
            "public", "writable/logs", "writable/cache",
            "tests",
        ] {
            std::fs::create_dir_all(project_dir.join(d))?;
        }

        std::fs::write(project_dir.join("composer.json"),
            klyron_template::TemplateEngine::render_static(
                &format!(r#"{{
  "name": "app/{{ name }}",
  "type": "project",
  "description": "CodeIgniter 4 project",
  "require": {{
    "php": "{}",
    "codeigniter4/framework": "{}"
  }},
  "require-dev": {{
    "phpunit/phpunit": "^10.0",
    "phpstan/phpstan": "^1.0"
  }},
  "autoload": {{
    "psr-4": {{ "App\\": "app/" }}
  }},
  "scripts": {{
    "post-create-project-cmd": [
      "php spark key:generate --ansi"
    ]
  }}
}}"#, php_req, version),
            vars))?;

        std::fs::write(project_dir.join(".env"),
            r#"CI_ENVIRONMENT = development
app.baseURL = 'http://localhost:8080'
database.default.hostname = localhost
database.default.database = ci4
database.default.username = root
database.default.password =
database.default.DBDriver = MySQLi
"#)?;

        std::fs::write(project_dir.join(".gitignore"),
            "/vendor\n.env\n/writable/*\n!writable/logs/.gitkeep\n!writable/cache/.gitkeep\n.DS_Store\n")?;

        std::fs::write(project_dir.join("spark"),
            r#"#!/usr/bin/env php
<?php
define('SPARKED', true);
require __DIR__ . '/system/Commands/Spark.php';
"#)?;

        std::fs::write(project_dir.join("public/index.php"),
            r#"<?php
require __DIR__ . '/../app/Config/Paths.php';
require __DIR__ . '/../vendor/codeigniter4/framework/system/Bootstrap.php';
"#)?;

        std::fs::write(project_dir.join("app/Config/Paths.php"),
            r#"<?php
namespace Config;
class Paths
{
    public string $systemDirectory = __DIR__ . '/../vendor/codeigniter4/framework/system';
    public string $appDirectory = __DIR__ . '/..';
    public string $writableDirectory = __DIR__ . '/../writable';
    public string $testsDirectory = __DIR__ . '/../tests';
    public string $viewDirectory = __DIR__ . '/../app/Views';
}
"#)?;

        std::fs::write(project_dir.join("app/Config/App.php"),
            r#"<?php
namespace Config;
use CodeIgniter\Config\BaseConfig;
class App extends BaseConfig
{
    public string $baseURL = 'http://localhost:8080';
    public string $indexPage = '';
    public string $uriProtocol = 'REQUEST_URI';
    public string $defaultLocale = 'en';
    public bool $negotiateLocale = false;
    public array $supportedLocales = ['en'];
    public string $appTimezone = 'UTC';
    public string $charset = 'UTF-8';
    public bool $forceGlobalSecureRequests = false;
    public array $proxyIPs = [];
}
"#)?;

        std::fs::write(project_dir.join("app/Config/Database.php"),
            r#"<?php
namespace Config;
use CodeIgniter\Database\Config;
class Database extends Config
{
    public array $default = [
        'DSN'      => '',
        'hostname' => 'localhost',
        'username' => 'root',
        'password' => '',
        'database' => 'ci4',
        'DBDriver' => 'MySQLi',
        'DBPrefix' => '',
        'port'     => 3306,
        'charset'  => 'utf8',
        'DBCollat' => 'utf8_general_ci',
    ];
}
"#)?;

        std::fs::write(project_dir.join("app/Config/Routes.php"),
            r#"<?php
use CodeIgniter\Router\RouteCollection;
/** @var RouteCollection $routes */
$routes->get('/', 'Home::index');
"#)?;

        std::fs::write(project_dir.join("app/Controllers/BaseController.php"),
            r#"<?php
namespace App\Controllers;
use CodeIgniter\Controller;
class BaseController extends Controller
{
    protected array $helpers = [];
}
"#)?;

        std::fs::write(project_dir.join("app/Controllers/Home.php"),
            r#"<?php
namespace App\Controllers;
class Home extends BaseController
{
    public function index(): string
    {
        return view('welcome');
    }
}
"#)?;

        std::fs::write(project_dir.join("app/Models/UserModel.php"),
            r#"<?php
namespace App\Models;
use CodeIgniter\Model;
class UserModel extends Model
{
    protected string $table = 'users';
    protected string $primaryKey = 'id';
    protected array $allowedFields = ['name', 'email', 'password'];
    protected bool $useTimestamps = true;
    protected array $validationRules = [
        'name'  => 'required|min_length[3]',
        'email' => 'required|valid_email',
    ];
}
"#)?;

        std::fs::write(project_dir.join("app/Views/welcome.php"),
            klyron_template::TemplateEngine::render_static(
                r#"<!DOCTYPE html>
<html><head><meta charset="UTF-8"><title><?= esc('{{ name }}') ?></title></head>
<body><h1>Welcome to <?= esc('{{ name }}') ?></h1></body>
</html>"#, vars))?;

        std::fs::write(project_dir.join("phpunit.xml.dist"),
            r#"<?xml version="1.0" encoding="UTF-8"?>
<phpunit bootstrap="tests/bootstrap.php" colors="true">
    <testsuites><testsuite name="Unit"><directory>tests</directory></testsuite></testsuites>
    <php><env name="CI_ENVIRONMENT" value="testing"/></php>
</phpunit>
"#)?;

        std::fs::write(project_dir.join("tests/bootstrap.php"),
            r#"<?php
require __DIR__ . '/../vendor/autoload.php';
"#)?;

        println!("CodeIgniter {} app created: {}", version, project_dir.display());
        Ok(())
    }
}
