import { readFileSync, statSync, writeFileSync, mkdirSync } from 'fs'
import { join } from 'path'

export function createAdapter() {
  return {
    name: 'flask',
    detect(dir) {
      try {
        const req = readFileSync(join(dir, 'requirements.txt'), 'utf-8').toLowerCase()
        if (req.includes('flask')) return true
      } catch {}
      try {
        const pyproject = readFileSync(join(dir, 'pyproject.toml'), 'utf-8').toLowerCase()
        if (pyproject.includes('flask')) return true
      } catch {}
      try {
        return statSync(join(dir, 'app.py')).isFile()
      } catch {}
      return false
    },
    supportedVersions: ['3.1'],
    defaultVersion: '3.1',
    kind: 'Polyglot',

    async dev(dir, port) {
      const { spawn } = await import('child_process')
      const proc = spawn('flask', ['--app', 'app.py', 'run', '--host', '0.0.0.0', '--port', String(port || 5000)], { cwd: dir, stdio: 'inherit', env: { ...process.env, FLASK_ENV: 'development' } })
      return new Promise((resolve, reject) => {
        proc.on('close', (code) => code === 0 ? resolve() : reject(new Error(`Dev server exited with code ${code}`)))
      })
    },

    async build(dir) {
      const { execFile } = await import('child_process')
      await execFile('pip', ['install', '-r', 'requirements.txt'], { cwd: dir, stdio: 'inherit' })
    },

    async test(dir) {
      const { execFile } = await import('child_process')
      await execFile('python3', ['-m', 'pytest'], { cwd: dir, stdio: 'inherit' })
    },

    async lint(dir) {
      const { execFile } = await import('child_process')
      await execFile('python3', ['-m', 'ruff', 'check', '.'], { cwd: dir, stdio: 'inherit' })
    },

    async format(dir, writeMode) {
      const { execFile } = await import('child_process')
      const args = writeMode ? ['-m', 'ruff', 'format', '.'] : ['-m', 'ruff', 'format', '--check', '.']
      await execFile('python3', args, { cwd: dir, stdio: 'inherit' })
    },

    scaffold(name, options) {
      return scaffoldFlask(name, options)
    }
  }
}

function existsSync(p) {
  try { return statSync(p).isFile() } catch { return false }
}

async function scaffoldFlask(name, options) {
  const projectDir = join(options.dir || process.cwd(), name)

  mkdirSync(join(projectDir, 'app', 'routes'), { recursive: true })
  mkdirSync(join(projectDir, 'app', 'models'), { recursive: true })
  mkdirSync(join(projectDir, 'app', 'templates'), { recursive: true })
  mkdirSync(join(projectDir, 'app', 'static', 'css'), { recursive: true })
  mkdirSync(join(projectDir, 'tests'), { recursive: true })

  writeFileSync(join(projectDir, 'requirements.txt'), `flask==3.1.0
flask-sqlalchemy==3.1.1
flask-migrate==4.0.7
flask-cors==5.0.0
python-dotenv==1.0.1
pytest==8.3.4
pytest-cov==6.0.0
ruff==0.8.4
`)

  writeFileSync(join(projectDir, '.env'), `FLASK_APP=app.py
FLASK_ENV=development
DATABASE_URL=sqlite:///${name}.db
SECRET_KEY=change-me-to-a-random-secret-key
`)

  writeFileSync(join(projectDir, '.gitignore'), `__pycache__/
*.py[cod]
*.pyo
.env
venv/
.venv/
*.db
.DS_Store
`)

  writeFileSync(join(projectDir, 'app.py'), `from app import create_app

app = create_app()

if __name__ == "__main__":
    app.run(host="0.0.0.0", port=5000, debug=True)
`)

  writeFileSync(join(projectDir, 'app', '__init__.py'), `from flask import Flask
from flask_sqlalchemy import SQLAlchemy
from flask_migrate import Migrate
from flask_cors import CORS
from dotenv import load_dotenv
import os

load_dotenv()

db = SQLAlchemy()
migrate = Migrate()


def create_app():
    app = Flask(__name__)
    app.config["SECRET_KEY"] = os.getenv("SECRET_KEY", "dev-key")
    app.config["SQLALCHEMY_DATABASE_URI"] = os.getenv("DATABASE_URL", "sqlite:///${name}.db")
    app.config["SQLALCHEMY_TRACK_MODIFICATIONS"] = False

    db.init_app(app)
    migrate.init_app(app, db)
    CORS(app)

    from app.routes.main_routes import main_bp
    from app.routes.api import api_bp

    app.register_blueprint(main_bp)
    app.register_blueprint(api_bp, url_prefix="/api")

    with app.app_context():
        import app.models
        db.create_all()

    return app
`)

  writeFileSync(join(projectDir, 'app', 'routes', '__init__.py'), ``)

  writeFileSync(join(projectDir, 'app', 'routes', 'main_routes.py'), `from flask import Blueprint, render_template

main_bp = Blueprint("main", __name__)


@main_bp.route("/")
def index():
    return render_template("index.html", title="Home")


@main_bp.route("/health")
def health():
    return {"status": "ok", "service": "${name}"}
`)

  writeFileSync(join(projectDir, 'app', 'routes', 'api.py'), `from flask import Blueprint, jsonify, request
from app import db
from app.models.item import Item
from app.models.user import User

api_bp = Blueprint("api", __name__)


@api_bp.route("/items", methods=["GET"])
def list_items():
    items = Item.query.all()
    return jsonify([item.to_dict() for item in items])


@api_bp.route("/items", methods=["POST"])
def create_item():
    data = request.get_json()
    item = Item(name=data["name"], description=data.get("description"), price=data["price"])
    db.session.add(item)
    db.session.commit()
    return jsonify(item.to_dict()), 201


@api_bp.route("/items/<int:item_id>", methods=["GET"])
def get_item(item_id):
    item = Item.query.get_or_404(item_id)
    return jsonify(item.to_dict())


@api_bp.route("/items/<int:item_id>", methods=["PUT"])
def update_item(item_id):
    item = Item.query.get_or_404(item_id)
    data = request.get_json()
    item.name = data.get("name", item.name)
    item.description = data.get("description", item.description)
    item.price = data.get("price", item.price)
    db.session.commit()
    return jsonify(item.to_dict())


@api_bp.route("/items/<int:item_id>", methods=["DELETE"])
def delete_item(item_id):
    item = Item.query.get_or_404(item_id)
    db.session.delete(item)
    db.session.commit()
    return "", 204


@api_bp.route("/users", methods=["GET"])
def list_users():
    users = User.query.all()
    return jsonify([user.to_dict() for user in users])


@api_bp.route("/users", methods=["POST"])
def create_user():
    data = request.get_json()
    if User.query.filter_by(email=data["email"]).first():
        return jsonify({"error": "Email already registered"}), 409
    user = User(name=data["name"], email=data["email"])
    db.session.add(user)
    db.session.commit()
    return jsonify(user.to_dict()), 201
`)

  writeFileSync(join(projectDir, 'app', 'models', '__init__.py'), `from app.models.item import Item
from app.models.user import User

__all__ = ["Item", "User"]
`)

  writeFileSync(join(projectDir, 'app', 'models', 'item.py'), `from app import db


class Item(db.Model):
    __tablename__ = "items"

    id = db.Column(db.Integer, primary_key=True)
    name = db.Column(db.String(255), nullable=False)
    description = db.Column(db.Text, nullable=True)
    price = db.Column(db.Float, nullable=False)

    def to_dict(self):
        return {
            "id": self.id,
            "name": self.name,
            "description": self.description,
            "price": self.price,
        }
`)

  writeFileSync(join(projectDir, 'app', 'models', 'user.py'), `from app import db


class User(db.Model):
    __tablename__ = "users"

    id = db.Column(db.Integer, primary_key=True)
    name = db.Column(db.String(255), nullable=False)
    email = db.Column(db.String(255), unique=True, nullable=False)

    def to_dict(self):
        return {
            "id": self.id,
            "name": self.name,
            "email": self.email,
        }
`)

  writeFileSync(join(projectDir, 'app', 'templates', 'base.html'), `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{ title if title else '${name}' }}</title>
    <link rel="stylesheet" href="{{ url_for('static', filename='css/style.css') }}">
</head>
<body>
    <nav>
        <h1><a href="/">{{ '${name}' }}</a></h1>
    </nav>
    <main>
        {% block content %}{% endblock %}
    </main>
</body>
</html>
`)

  writeFileSync(join(projectDir, 'app', 'templates', 'index.html'), `{% extends "base.html" %}
{% block content %}
<section class="hero">
    <h2>Welcome to ${name}</h2>
    <p>A Flask application.</p>
    <a href="/health" class="btn">Health Check</a>
</section>
{% endblock %}
`)

  writeFileSync(join(projectDir, 'app', 'static', 'css', 'style.css'), `* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    line-height: 1.6;
    color: #333;
}

nav {
    background: #1a1a2e;
    padding: 1rem 2rem;
}

nav h1 a {
    color: #fff;
    text-decoration: none;
    font-size: 1.5rem;
}

main {
    padding: 2rem;
    max-width: 960px;
    margin: 0 auto;
}

.hero {
    text-align: center;
    padding: 4rem 0;
}

.hero h2 {
    font-size: 2.5rem;
    margin-bottom: 1rem;
}

.btn {
    display: inline-block;
    padding: 0.75rem 1.5rem;
    background: #1a1a2e;
    color: #fff;
    text-decoration: none;
    border-radius: 4px;
    margin-top: 1rem;
}
`)

  writeFileSync(join(projectDir, 'tests', '__init__.py'), ``)

  writeFileSync(join(projectDir, 'tests', 'test_app.py'), `import pytest
from app import create_app


@pytest.fixture
def app():
    app = create_app()
    app.config["TESTING"] = True
    app.config["SQLALCHEMY_DATABASE_URI"] = "sqlite:///:memory:"
    return app


@pytest.fixture
def client(app):
    return app.test_client()


def test_health(client):
    resp = client.get("/health")
    assert resp.status_code == 200
    assert resp.json["status"] == "ok"


def test_create_item(client):
    resp = client.post("/api/items", json={"name": "Test", "description": "A test item", "price": 9.99})
    assert resp.status_code == 201
    data = resp.get_json()
    assert data["name"] == "Test"
    assert data["price"] == 9.99
`)

  writeFileSync(join(projectDir, 'README.md'), `# ${name}

Flask application

## Getting Started

\`\`\`bash
pip install -r requirements.txt
flask --app app.py run --host 0.0.0.0 --port 5000
\`\`\`
`)
}
