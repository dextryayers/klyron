use std::path::Path;
use async_trait::async_trait;
use anyhow::Result;
use crate::{FrameworkAdapter, BuildOptions, ScaffoldOptions, FrameworkKind};

pub struct SymfonyAdapter;

fn version_deps(version: &str) -> (&'static str, &'static str, &'static str, &'static str) {
    match version {
        "5.4" => ("5.4.*", "^7.4|^8.0", "^5.4", "^9.5"),
        "6.4" => ("6.4.*", "^8.1", "^6.4", "^10.5"),
        "7.0" => ("7.0.*", "^8.2", "^7.0", "^11.0"),
        "7.1" => ("7.1.*", "^8.2", "^7.1", "^11.0"),
        "7.2" => ("7.2.*", "^8.3", "^7.2", "^11.5"),
        _ => ("7.1.*", "^8.2", "^7.1", "^11.0"),
    }
}

#[async_trait]
impl FrameworkAdapter for SymfonyAdapter {
    fn name(&self) -> &'static str { "symfony" }

    fn detect(&self, dir: &Path) -> bool {
        let composer = dir.join("composer.json");
        if let Ok(content) = std::fs::read_to_string(composer) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(require) = json.get("require") {
                    for key in &["symfony/framework-bundle", "symfony/symfony", "symfony/http-kernel"] {
                        if require.get(*key).is_some() { return true; }
                    }
                }
            }
        }
        dir.join("bin/console").exists() && dir.join("config").exists()
    }

    fn supported_versions(&self) -> Vec<&'static str> { vec!["5.4", "6.4", "7.0", "7.1", "7.2"] }
    fn default_version(&self) -> &'static str { "7.1" }
    fn kind(&self) -> FrameworkKind { FrameworkKind::Backend }

    async fn dev(&self, dir: &Path, port: Option<u16>) -> Result<()> {
        let mut cmd = tokio::process::Command::new("symfony");
        cmd.args(["server:start"]).current_dir(dir);
        if let Some(p) = port { cmd.arg(format!("--port={}", p)); }
        cmd.status().await?; Ok(())
    }

    async fn build(&self, _dir: &Path, _opts: BuildOptions) -> Result<()> { Ok(()) }

    async fn test(&self, dir: &Path, _filter: Option<&str>) -> Result<()> {
        tokio::process::Command::new("php")
            .args(["./bin/phpunit"])
            .current_dir(dir)
            .status().await?;
        Ok(())
    }

    async fn lint(&self, dir: &Path, _fix: bool) -> Result<()> {
        tokio::process::Command::new("php")
            .args(["./vendor/bin/php-cs-fixer", "fix", "--dry-run"])
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
                .args(["./vendor/bin/php-cs-fixer", "fix", "--dry-run"])
                .current_dir(dir).status().await?;
        }
        Ok(())
    }

    fn external_scaffold_command(&self, name: &str, version: Option<&str>) -> Option<(String, Vec<String>)> {
        let mut args = vec!["create-project".into(), "symfony/skeleton".into(), name.into()];
        if let Some(v) = version {
            args.push(v.into());
        }
        Some(("composer".into(), args))
    }

    async fn scaffold(&self, name: &str, options: ScaffoldOptions) -> Result<()> {
        if options.external {
            if let Some((cmd, args)) = self.external_scaffold_command(name, options.version.as_deref()) {
                let status = std::process::Command::new(&cmd).args(&args).current_dir(&options.dir).status()?;
                if !status.success() { anyhow::bail!("External scaffolding failed"); }
                return Ok(());
            }
        }

        let version = options.version.as_deref().unwrap_or("7.1");
        let (fw_dep, php_req, _f, _p) = version_deps(version);
        let vars = &options.template_vars;
        let project_dir = options.dir.join(name);

        for d in &[
            "src/Controller", "src/Entity", "src/Repository", "src/Service",
            "src/Form", "src/EventListener", "src/DataFixtures",
            "config/packages", "config/routes",
            "templates", "translations",
            "public", "migrations", "tests",
            "var/log",
        ] {
            std::fs::create_dir_all(project_dir.join(d))?;
        }

        std::fs::write(project_dir.join("composer.json"),
            klyron_template::TemplateEngine::render_static(
                &format!(r#"{{
  "name": "app/{{ name }}",
  "type": "project",
  "description": "Symfony project",
  "require": {{
    "php": "{}",
    "symfony/framework-bundle": "{}",
    "symfony/dotenv": "^7.0",
    "symfony/yaml": "^7.0",
    "symfony/twig-bundle": "^7.0",
    "symfony/orm-pack": "^2.0",
    "symfony/serializer-pack": "^1.0",
    "doctrine/doctrine-migrations-bundle": "^3.3"
  }},
  "require-dev": {{
    "symfony/debug-bundle": "^7.0",
    "symfony/maker-bundle": "^1.0",
    "symfony/phpunit-bridge": "^7.0",
    "phpunit/phpunit": "^11.0",
    "doctrine/doctrine-fixtures-bundle": "^3.5"
  }},
  "autoload": {{
    "psr-4": {{ "App\\": "src/" }}
  }},
  "scripts": {{
    "auto-scripts": {{
      "cache:clear": "symfony-cmd",
      "assets:install %PUBLIC_DIR%": "symfony-cmd"
    }},
    "post-install-cmd": [
      "@auto-scripts"
    ],
    "post-update-cmd": [
      "@auto-scripts"
    ]
  }},
  "extra": {{
    "symfony": {{ "allow-contrib": false }}
  }}
}}"#, php_req, fw_dep),
            vars))?;

        std::fs::write(project_dir.join(".env"),
            r#"APP_ENV=dev
APP_SECRET=change-this-secret-key
DATABASE_URL=sqlite:///%kernel.project_dir%/var/data.db
TRUSTED_PROXIES=127.0.0.1,::1
TRUSTED_HOSTS='^localhost|example\.com$'
"#)?;

        std::fs::write(project_dir.join(".env.example"),
            r#"APP_ENV=dev
APP_SECRET=change-this-secret-key
DATABASE_URL=sqlite:///%kernel.project_dir%/var/data.db
"#)?;

        std::fs::write(project_dir.join(".gitignore"),
            "/vendor\n.env\n/var/*\n!var/log\n.DS_Store\n")?;

        std::fs::write(project_dir.join("public/index.php"),
            r#"<?php
use App\Kernel;
require_once dirname(__DIR__).'/vendor/autoload_runtime.php';
return function (array $context) {
    return new Kernel($context['APP_ENV'], (bool) $context['APP_DEBUG']);
};
"#)?;

        std::fs::write(project_dir.join("src/Kernel.php"),
            klyron_template::TemplateEngine::render_static(r#"<?php
namespace App;
use Symfony\Bundle\FrameworkBundle\Kernel\MicroKernelTrait;
use Symfony\Component\HttpKernel\Kernel as BaseKernel;
class Kernel extends BaseKernel
{
    use MicroKernelTrait;
}
"#, vars))?;

        std::fs::write(project_dir.join("src/Controller/DefaultController.php"),
            r#"<?php
namespace App\Controller;
use Symfony\Bundle\FrameworkBundle\Controller\AbstractController;
use Symfony\Component\HttpFoundation\Response;
use Symfony\Component\Routing\Annotation\Route;
class DefaultController extends AbstractController
{
    #[Route('/', name: 'home')]
    public function index(): Response
    {
        return $this->render('base.html.twig', ['title' => 'Welcome']);
    }
}
"#)?;

        std::fs::write(project_dir.join("templates/base.html.twig"),
            klyron_template::TemplateEngine::render_static(
                r#"<!DOCTYPE html>
<html>
<head><meta charset="UTF-8"><title>{% block title %}{{ name }}{% endblock %}</title>{% block stylesheets %}{% endblock %}</head>
<body>{% block body %}<h1>Welcome to {{ name }}</h1>{% endblock %}{% block javascripts %}{% endblock %}</body>
</html>"#, vars))?;

        std::fs::write(project_dir.join("config/packages/framework.yaml"),
            r#"framework:
    secret: '%env(APP_SECRET)%'
    http_method_override: true
    handle_all_throwables: true
    session:
        handler_id: null
        cookie_secure: auto
        cookie_samesite: lax
    php_errors:
        log: true
"#)?;

        std::fs::write(project_dir.join("config/packages/doctrine.yaml"),
            r#"doctrine:
    dbal:
        url: '%env(DATABASE_URL)%'
    orm:
        auto_generate_proxy_classes: true
        naming_strategy: doctrine.orm.naming_strategy.underscore_number_aware
        auto_mapping: true
        mappings:
            App:
                is_bundle: false
                dir: '%kernel.project_dir%/src/Entity'
                prefix: 'App\Entity'
                alias: App
"#)?;

        std::fs::write(project_dir.join("config/packages/twig.yaml"),
            r#"twig:
    default_path: '%kernel.project_dir%/templates'
"#)?;

        std::fs::write(project_dir.join("config/services.yaml"),
            r#"services:
    _defaults:
        autowire: true
        autoconfigure: true
    App\:
        resource: '../src/'
        exclude:
            - '../src/DependencyInjection/'
            - '../src/Entity/'
            - '../src/Kernel.php'
"#)?;

        std::fs::write(project_dir.join("config/routes/annotations.yaml"),
            r#"controllers:
    resource: ../../src/Controller/
    type: annotation
"#)?;

        std::fs::write(project_dir.join("config/bundles.php"),
            r#"<?php
return [
    Symfony\Bundle\FrameworkBundle\FrameworkBundle::class => ['all' => true],
    Symfony\Bundle\TwigBundle\TwigBundle::class => ['all' => true],
    Symfony\Bundle\MakerBundle\MakerBundle::class => ['dev' => true],
    Symfony\Bundle\DebugBundle\DebugBundle::class => ['dev' => true],
    Doctrine\Bundle\DoctrineBundle\DoctrineBundle::class => ['all' => true],
    Doctrine\Bundle\MigrationsBundle\DoctrineMigrationsBundle::class => ['all' => true],
    Doctrine\Bundle\FixturesBundle\DoctrineFixturesBundle::class => ['dev' => true, 'test' => true],
];
"#)?;

        std::fs::write(project_dir.join("src/Entity/User.php"),
            r#"<?php
namespace App\Entity;
use Doctrine\ORM\Mapping as ORM;
#[ORM\Entity(repositoryClass: 'App\Repository\UserRepository')]
#[ORM\Table(name: '`user`')]
class User
{
    #[ORM\Id, ORM\GeneratedValue, ORM\Column(type: 'integer')]
    private int $id;
    #[ORM\Column(type: 'string', length: 180, unique: true)]
    private string $email;
    #[ORM\Column(type: 'string')]
    private string $password;
    public function getId(): int { return $this->id; }
    public function getEmail(): string { return $this->email; }
    public function setEmail(string $email): self { $this->email = $email; return $this; }
    public function getPassword(): string { return $this->password; }
    public function setPassword(string $password): self { $this->password = $password; return $this; }
}
"#)?;

        std::fs::write(project_dir.join("src/Repository/UserRepository.php"),
            r#"<?php
namespace App\Repository;
use App\Entity\User;
use Doctrine\Bundle\DoctrineBundle\Repository\ServiceEntityRepository;
use Doctrine\Persistence\ManagerRegistry;
class UserRepository extends ServiceEntityRepository
{
    public function __construct(ManagerRegistry $registry)
    {
        parent::__construct($registry, User::class);
    }
}
"#)?;

        std::fs::write(project_dir.join("src/DataFixtures/AppFixtures.php"),
            r#"<?php
namespace App\DataFixtures;
use Doctrine\Bundle\FixturesBundle\Fixture;
use Doctrine\Persistence\ObjectManager;
class AppFixtures extends Fixture
{
    public function load(ObjectManager $manager): void
    {
        // Add fixtures here
        $manager->flush();
    }
}
"#)?;

        std::fs::write(project_dir.join("phpunit.xml.dist"),
            r#"<?xml version="1.0" encoding="UTF-8"?>
<phpunit xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
         xsi:noNamespaceSchemaLocation="vendor/phpunit/phpunit/phpunit.xsd"
         bootstrap="tests/bootstrap.php" colors="true">
    <testsuites><testsuite name="Unit"><directory>tests</directory></testsuite></testsuites>
    <php><env name="APP_ENV" value="test"/></php>
</phpunit>
"#)?;

        std::fs::write(project_dir.join("tests/bootstrap.php"),
            r#"<?php
require dirname(__DIR__).'/vendor/autoload.php';
"#)?;

        std::fs::write(project_dir.join("symfony.lock"),
            r#"# Auto-generated by Klyron. Remove if using composer directly.
"#)?;

        println!("Symfony {} app created: {} (PHP {})", version, project_dir.display(), php_req);
        Ok(())
    }
}
