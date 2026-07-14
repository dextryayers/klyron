use std::path::Path;
use async_trait::async_trait;
use anyhow::Result;
use crate::{FrameworkAdapter, BuildOptions, ScaffoldOptions, FrameworkKind};

pub struct WordPressAdapter;

fn version_wp(version: &str) -> &'static str {
    match version {
        "6.0" => "6.0.9",
        "6.1" => "6.1.7",
        "6.2" => "6.2.6",
        "6.3" => "6.3.5",
        "6.4" => "6.4.5",
        "6.5" => "6.5.5",
        "6.6" => "6.6.2",
        "6.7" => "6.7.2",
        _ => "6.7.2",
    }
}

#[async_trait]
impl FrameworkAdapter for WordPressAdapter {
    fn name(&self) -> &'static str { "wordpress" }

    fn detect(&self, dir: &Path) -> bool {
        dir.join("wp-config.php").exists() || dir.join("wp-includes").exists() || dir.join("wp-content").exists()
    }

    fn supported_versions(&self) -> Vec<&'static str> {
        vec!["6.0", "6.1", "6.2", "6.3", "6.4", "6.5", "6.6", "6.7"]
    }
    fn default_version(&self) -> &'static str { "6.7" }
    fn kind(&self) -> FrameworkKind { FrameworkKind::Fullstack }

    async fn dev(&self, dir: &Path, port: Option<u16>) -> Result<()> {
        let mut cmd = tokio::process::Command::new("php");
        cmd.args(["-S", format!("localhost:{}", port.unwrap_or(8000)).as_str(), "-t", dir.to_str().unwrap_or(".")])
            .current_dir(dir);
        cmd.status().await?; Ok(())
    }

    async fn build(&self, _dir: &Path, _opts: BuildOptions) -> Result<()> { Ok(()) }

    async fn test(&self, _dir: &Path, _filter: Option<&str>) -> Result<()> {
        println!("WordPress testing: use `wp scaffold plugin-tests` for plugin testing.");
        Ok(())
    }

    async fn lint(&self, _dir: &Path, _fix: bool) -> Result<()> { Ok(()) }

    async fn format(&self, _dir: &Path, _write: bool) -> Result<()> { Ok(()) }

    fn external_scaffold_command(&self, _name: &str, _version: Option<&str>) -> Option<(String, Vec<String>)> {
        None
    }

    async fn scaffold(&self, name: &str, options: ScaffoldOptions) -> Result<()> {
        let version = options.version.as_deref().unwrap_or("6.7");
        let wp_ver = version_wp(version);
        let vars = &options.template_vars;
        let project_dir = options.dir.join(name);

        for d in &[
            "wp-content/themes/default", "wp-content/plugins",
            "wp-content/uploads", "wp-content/languages",
            "wp-includes",
        ] {
            std::fs::create_dir_all(project_dir.join(d))?;
        }

        std::fs::write(project_dir.join("wp-config.php"),
            klyron_template::TemplateEngine::render_static(r#"<?php
define('DB_NAME', getenv('DB_NAME') ?: 'wordpress');
define('DB_USER', getenv('DB_USER') ?: 'root');
define('DB_PASSWORD', getenv('DB_PASSWORD') ?: '');
define('DB_HOST', getenv('DB_HOST') ?: 'localhost');
define('DB_CHARSET', 'utf8');
define('DB_COLLATE', '');
$table_prefix = 'wp_';
define('WP_DEBUG', true);
define('WP_DEBUG_LOG', true);
define('WP_DEBUG_DISPLAY', false);
if (!defined('ABSPATH')) define('ABSPATH', __DIR__ . '/');
require_once ABSPATH . 'wp-settings.php';
"#, vars))?;

        std::fs::write(project_dir.join(".htaccess"),
            r#"<IfModule mod_rewrite.c>
RewriteEngine On
RewriteRule .* - [E=HTTP_AUTHORIZATION:%{HTTP:Authorization}]
RewriteBase /
RewriteRule ^index\.php$ - [L]
RewriteCond %{REQUEST_FILENAME} !-f
RewriteCond %{REQUEST_FILENAME} !-d
RewriteRule . /index.php [L]
</IfModule>
"#)?;

        std::fs::write(project_dir.join("index.php"),
            klyron_template::TemplateEngine::render_static(r#"<?php
define('WP_USE_THEMES', true);
require __DIR__ . '/wp-blog-header.php';
"#, vars))?;

        std::fs::write(project_dir.join("wp-content/themes/default/style.css"),
            klyron_template::TemplateEngine::render_static(
                r#"/*
Theme Name: {{ name }} Default
Author: Klyron
Description: Default theme for {{ name }}
Version: 1.0.0
*/
body { font-family: system-ui, sans-serif; margin: 0; padding: 2rem; line-height: 1.6; color: #333; }
"#, vars))?;

        std::fs::write(project_dir.join("wp-content/themes/default/index.php"),
            r#"<!DOCTYPE html>
<html <?php language_attributes(); ?>>
<head><meta charset="<?php bloginfo('charset'); ?>"><?php wp_head(); ?></head>
<body <?php body_class(); ?>>
<header><h1><a href="<?php echo esc_url(home_url('/')); ?>"><?php bloginfo('name'); ?></a></h1></header>
<main><?php if(have_posts()): while(have_posts()): the_post(); ?><article><h2><?php the_title(); ?></h2><div><?php the_content(); ?></div></article><?php endwhile; endif; ?></main>
<footer><p>&copy; <?php echo date('Y'); ?> <?php bloginfo('name'); ?></p></footer>
<?php wp_footer(); ?></body></html>"#)?;

        std::fs::write(project_dir.join("wp-content/themes/default/functions.php"),
            r#"<?php
function klyron_setup(): void {
    add_theme_support('title-tag');
    add_theme_support('post-thumbnails');
    add_theme_support('html5', ['search-form', 'comment-form', 'comment-list', 'gallery', 'caption']);
    register_nav_menus(['primary' => __('Primary Menu', 'klyron')]);
}
add_action('after_setup_theme', 'klyron_setup');
function klyron_scripts(): void {
    wp_enqueue_style('klyron-style', get_stylesheet_uri());
}
add_action('wp_enqueue_scripts', 'klyron_scripts');
"#)?;

        std::fs::write(project_dir.join("wp-content/plugins/klyron-settings/klyron-settings.php"),
            r#"<?php
/**
 * Plugin Name: Klyron Settings
 * Description: Basic settings plugin scaffolded by Klyron.
 * Version: 1.0.0
 */
defined('ABSPATH') || exit;
"#)?;

        let composer_json = format!(r#"{{
  "name": "app/{}",
  "type": "wordpress-project",
  "description": "WordPress project",
  "require": {{
    "php": ">=8.0"
  }},
  "scripts": {{
    "post-install-cmd": [
      "wp core download --version={} --force"
    ]
  }}
}}"#, name, wp_ver);
        std::fs::write(project_dir.join("composer.json"), composer_json)?;

        std::fs::write(project_dir.join(".env.example"),
            r#"DB_NAME=wordpress
DB_USER=root
DB_PASSWORD=
DB_HOST=localhost
WP_HOME=http://localhost:8000
WP_SITEURL=http://localhost:8000
"#)?;

        std::fs::write(project_dir.join(".gitignore"),
            "/wp-content/uploads/*\n!wp-content/uploads/.gitkeep\nwp-config.php\n.env\n.DS_Store\n")?;

        let mut readme_vars = vars.clone();
        readme_vars.insert("version".to_string(), version.to_string());
        let readme = format!(r#"<!DOCTYPE html>
<html><head><title>{name} - WordPress</title></head>
<body><h1>{name}</h1><p>WordPress {version} project.</p></body>
</html>"#);
        std::fs::write(project_dir.join("readme.html"), readme)?;

        println!("WordPress {} app created: {} (WordPress {})", version, project_dir.display(), wp_ver);
        println!("\n  cd {}", project_dir.display());
        println!("  wp core download --version={}", wp_ver);
        println!("  wp core config --dbname=wordpress --dbuser=root");
        println!("  wp db create && wp core install --url=localhost --title=\"{}\" --admin_user=admin --admin_password=admin --admin_email=admin@example.com", name);
        Ok(())
    }
}
