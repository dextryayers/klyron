use std::path::Path;
use async_trait::async_trait;
use anyhow::Result;
use crate::{FrameworkAdapter, BuildOptions, ScaffoldOptions, FrameworkKind};

pub struct FlaskAdapter;

#[async_trait]
impl FrameworkAdapter for FlaskAdapter {
    fn name(&self) -> &'static str { "flask" }

    fn detect(&self, dir: &Path) -> bool {
        let pyproject = dir.join("pyproject.toml");
        if let Ok(content) = std::fs::read_to_string(pyproject) {
            if content.contains("flask") { return true; }
        }
        let requirements = dir.join("requirements.txt");
        if let Ok(content) = std::fs::read_to_string(requirements) {
            if content.to_lowercase().contains("flask") { return true; }
        }
        let app = dir.join("app.py");
        if let Ok(content) = std::fs::read_to_string(app) {
            if content.contains("from flask") || content.contains("import flask") { return true; }
        }
        false
    }

    fn supported_versions(&self) -> Vec<&'static str> { vec!["3.1"] }
    fn default_version(&self) -> &'static str { "3.1" }
    fn kind(&self) -> FrameworkKind { FrameworkKind::Backend }

    async fn dev(&self, dir: &Path, port: Option<u16>) -> Result<()> {
        let port = port.unwrap_or(5000);
        let mut cmd = tokio::process::Command::new("flask");
        cmd.args(["--app", "app.py", "run", "--host", "0.0.0.0", &format!("--port={}", port)])
            .current_dir(dir);
        cmd.status().await?; Ok(())
    }

    async fn build(&self, _dir: &Path, _opts: BuildOptions) -> Result<()> { Ok(()) }

    async fn test(&self, dir: &Path, _filter: Option<&str>) -> Result<()> {
        tokio::process::Command::new("python3")
            .args(["-m", "pytest", "."])
            .current_dir(dir)
            .status().await?;
        Ok(())
    }

    async fn lint(&self, dir: &Path, _fix: bool) -> Result<()> {
        tokio::process::Command::new("ruff")
            .args(["check", "."])
            .current_dir(dir).status().await?;
        Ok(())
    }

    async fn format(&self, dir: &Path, write: bool) -> Result<()> {
        if write {
            tokio::process::Command::new("ruff")
                .args(["format", "."])
                .current_dir(dir).status().await?;
        } else {
            tokio::process::Command::new("ruff")
                .args(["format", "--check", "."])
                .current_dir(dir).status().await?;
        }
        Ok(())
    }

    fn external_scaffold_command(&self, _name: &str, _version: Option<&str>) -> Option<(String, Vec<String>)> {
        Some(("python3".into(), vec!["-m".into(), "pip".into(), "install".into(), "flask".into()]))
    }

    async fn scaffold(&self, name: &str, options: ScaffoldOptions) -> Result<()> {
        let project_dir = options.dir.join(name);
        std::fs::create_dir_all(&project_dir)?;
        std::fs::create_dir_all(project_dir.join("app"))?;
        std::fs::create_dir_all(project_dir.join("app/routes"))?;
        std::fs::create_dir_all(project_dir.join("app/models"))?;
        std::fs::create_dir_all(project_dir.join("app/static"))?;
        std::fs::create_dir_all(project_dir.join("app/templates"))?;
        std::fs::create_dir_all(project_dir.join("tests"))?;

        let vars = &options.template_vars;

        std::fs::write(project_dir.join("app.py"),
            klyron_template::TemplateEngine::render_static(r#"from app import create_app

app = create_app()

if __name__ == "__main__":
    app.run(debug=True)
"#, vars))?;

        std::fs::write(project_dir.join("requirements.txt"),
            r#"flask>=3.1,<4.0
flask-sqlalchemy>=3.1,<4.0
flask-migrate>=4.0,<5.0
python-dotenv>=1.0,<2.0
pytest>=8.0,<9.0
ruff>=0.6,<1.0
"#)?;

        std::fs::write(project_dir.join("app/__init__.py"),
            klyron_template::TemplateEngine::render_static(r#"from flask import Flask
from flask_sqlalchemy import SQLAlchemy
from flask_migrate import Migrate

db = SQLAlchemy()
migrate = Migrate()


def create_app():
    app = Flask(__name__)
    app.config["SECRET_KEY"] = "change-me"
    app.config["SQLALCHEMY_DATABASE_URI"] = "sqlite:///app.db"

    db.init_app(app)
    migrate.init_app(app, db)

    from app.routes import main, items

    app.register_blueprint(main.bp)
    app.register_blueprint(items.bp, url_prefix="/api/items")

    return app
"#, vars))?;

        std::fs::write(project_dir.join("app/routes/__init__.py"), "")?;

        std::fs::write(project_dir.join("app/routes/main.py"),
            klyron_template::TemplateEngine::render_static(r#"from flask import Blueprint, jsonify

bp = Blueprint("main", __name__)


@bp.route("/")
def index():
    return jsonify({"message": "Welcome to {{ name }}"})
"#, vars))?;

        std::fs::write(project_dir.join("app/routes/items.py"),
            r#"from flask import Blueprint, jsonify, request
from app import db
from app.models.item import Item

bp = Blueprint("items", __name__)


@bp.route("/")
def list_items():
    items = Item.query.all()
    return jsonify([{"id": i.id, "name": i.name} for i in items])


@bp.route("/", methods=["POST"])
def create_item():
    data = request.get_json()
    item = Item(name=data["name"])
    db.session.add(item)
    db.session.commit()
    return jsonify({"id": item.id, "name": item.name}), 201
"#)?;

        std::fs::write(project_dir.join("app/models/__init__.py"), "")?;

        std::fs::write(project_dir.join("app/models/item.py"),
            r#"from app import db
from datetime import datetime


class Item(db.Model):
    id = db.Column(db.Integer, primary_key=True)
    name = db.Column(db.String(255), nullable=False)
    created_at = db.Column(db.DateTime, default=datetime.utcnow)

    def __repr__(self):
        return f"<Item {self.name}>"
"#)?;

        std::fs::write(project_dir.join("app/static/style.css"),
            r#"* { margin: 0; padding: 0; box-sizing: border-box; }
body { font-family: system-ui, sans-serif; }
"#)?;

        std::fs::write(project_dir.join("app/templates/base.html"),
            klyron_template::TemplateEngine::render_static(r#"<!doctype html>
<html><head><meta charset="utf-8"><title>{% block title %}{{ name }}{% endblock %}</title></head>
<body>{% block content %}{% endblock %}</body></html>
"#, vars))?;

        std::fs::write(project_dir.join("tests/__init__.py"), "")?;

        std::fs::write(project_dir.join("tests/conftest.py"),
            r#"import pytest
from app import create_app


@pytest.fixture
def app():
    app = create_app()
    app.config["TESTING"] = True
    yield app


@pytest.fixture
def client(app):
    return app.test_client()
"#)?;

        std::fs::write(project_dir.join("tests/test_app.py"),
            r##"def test_index(client):
    response = client.get("/")
    assert response.status_code == 200
    assert "message" in response.json
"##)?;

        std::fs::write(project_dir.join(".env"),
            r#"FLASK_APP=app.py
FLASK_DEBUG=True
SECRET_KEY=change-me
DATABASE_URL=sqlite:///app.db
"#)?;

        std::fs::write(project_dir.join(".env.example"),
            r#"FLASK_APP=app.py
FLASK_DEBUG=True
SECRET_KEY=change-me
DATABASE_URL=sqlite:///app.db
"#)?;

        std::fs::write(project_dir.join("pytest.ini"),
            r#"[pytest]
testpaths = tests
"#)?;

        std::fs::write(project_dir.join(".gitignore"),
            "*.pyc\n__pycache__\n.DS_Store\n*.db\n.env\ninstance\n")?;

        std::fs::write(project_dir.join("README.md"),
            klyron_template::TemplateEngine::render_static(r#"# {{ name }}

Flask project

## Getting Started

pip install -r requirements.txt
flask --app app.py run --debug
"#, vars))?;

        Ok(())
    }
}
