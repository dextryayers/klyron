use std::path::Path;
use async_trait::async_trait;
use anyhow::Result;
use crate::{FrameworkAdapter, BuildOptions, ScaffoldOptions, FrameworkKind};

pub struct FastApiAdapter;

#[async_trait]
impl FrameworkAdapter for FastApiAdapter {
    fn name(&self) -> &'static str { "fastapi" }

    fn detect(&self, dir: &Path) -> bool {
        let pyproject = dir.join("pyproject.toml");
        if let Ok(content) = std::fs::read_to_string(pyproject) {
            if content.contains("fastapi") { return true; }
        }
        let requirements = dir.join("requirements.txt");
        if let Ok(content) = std::fs::read_to_string(requirements) {
            if content.to_lowercase().contains("fastapi") { return true; }
        }
        let main = dir.join("main.py");
        if let Ok(content) = std::fs::read_to_string(main) {
            if content.contains("from fastapi") || content.contains("import fastapi") { return true; }
        }
        false
    }

    fn supported_versions(&self) -> Vec<&'static str> { vec!["0.115"] }
    fn default_version(&self) -> &'static str { "0.115" }
    fn kind(&self) -> FrameworkKind { FrameworkKind::Backend }

    async fn dev(&self, dir: &Path, port: Option<u16>) -> Result<()> {
        let port = port.unwrap_or(8000);
        let mut cmd = tokio::process::Command::new("uvicorn");
        cmd.args(["main:app", "--reload", "--host", "0.0.0.0", &format!("--port={}", port)])
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
        Some(("python3".into(), vec!["-m".into(), "pip".into(), "install".into(), "fastapi".into(), "uvicorn".into()]))
    }

    async fn scaffold(&self, name: &str, options: ScaffoldOptions) -> Result<()> {
        let project_dir = options.dir.join(name);
        std::fs::create_dir_all(&project_dir)?;
        std::fs::create_dir_all(project_dir.join("app/routers"))?;
        std::fs::create_dir_all(project_dir.join("app/models"))?;
        std::fs::create_dir_all(project_dir.join("app/schemas"))?;
        std::fs::create_dir_all(project_dir.join("app/services"))?;
        std::fs::create_dir_all(project_dir.join("tests"))?;

        let vars = &options.template_vars;

        std::fs::write(project_dir.join("main.py"),
            klyron_template::TemplateEngine::render_static(r#"from fastapi import FastAPI
from app.routers import items, users

app = FastAPI(title="{{ name }}", version="0.1.0")

app.include_router(items.router, prefix="/api/items", tags=["items"])
app.include_router(users.router, prefix="/api/users", tags=["users"])


@app.get("/")
async def root():
    return {"message": "Welcome to {{ name }}"}
"#, vars))?;

        std::fs::write(project_dir.join("requirements.txt"),
            r#"fastapi>=0.115.0,<0.116.0
uvicorn>=0.30.0,<1.0.0
sqlalchemy>=2.0,<3.0
alembic>=1.13,<2.0
pydantic>=2.0,<3.0
pydantic-settings>=2.0,<3.0
httpx>=0.27,<1.0
pytest>=8.0,<9.0
pytest-asyncio>=0.24,<1.0
ruff>=0.6,<1.0
"#)?;

        std::fs::write(project_dir.join("app/__init__.py"), "")?;
        std::fs::write(project_dir.join("app/routers/__init__.py"), "")?;
        std::fs::write(project_dir.join("app/models/__init__.py"), "")?;
        std::fs::write(project_dir.join("app/schemas/__init__.py"), "")?;
        std::fs::write(project_dir.join("app/services/__init__.py"), "")?;

        std::fs::write(project_dir.join("app/database.py"),
            r#"from sqlalchemy import create_engine
from sqlalchemy.orm import sessionmaker, DeclarativeBase

DATABASE_URL = "sqlite:///./app.db"

engine = create_engine(DATABASE_URL, connect_args={"check_same_thread": False})
SessionLocal = sessionmaker(autocommit=False, autoflush=False, bind=engine)


class Base(DeclarativeBase):
    pass


def get_db():
    db = SessionLocal()
    try:
        yield db
    finally:
        db.close()
"#)?;

        std::fs::write(project_dir.join("app/config.py"),
            klyron_template::TemplateEngine::render_static(r#"from pydantic_settings import BaseSettings


class Settings(BaseSettings):
    app_name: str = "{{ name }}"
    debug: bool = True
    database_url: str = "sqlite:///./app.db"

    class Config:
        env_file = ".env"


settings = Settings()
"#, vars))?;

        std::fs::write(project_dir.join("app/models/item.py"),
            r#"from sqlalchemy import Column, Integer, String, Text, DateTime, func
from app.database import Base


class Item(Base):
    __tablename__ = "items"

    id = Column(Integer, primary_key=True, index=True)
    name = Column(String(255), nullable=False)
    description = Column(Text, default="")
    created_at = Column(DateTime, server_default=func.now())
"#)?;

        std::fs::write(project_dir.join("app/schemas/item.py"),
            r#"from pydantic import BaseModel
from datetime import datetime
from typing import Optional


class ItemBase(BaseModel):
    name: str
    description: Optional[str] = None


class ItemCreate(ItemBase):
    pass


class ItemResponse(ItemBase):
    id: int
    created_at: datetime

    model_config = {"from_attributes": True}
"#)?;

        std::fs::write(project_dir.join("app/routers/items.py"),
            r#"from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy.orm import Session
from app.database import get_db
from app.models.item import Item
from app.schemas.item import ItemCreate, ItemResponse

router = APIRouter()


@router.get("/", response_model=list[ItemResponse])
async def list_items(db: Session = Depends(get_db)):
    return db.query(Item).all()


@router.post("/", response_model=ItemResponse, status_code=201)
async def create_item(item: ItemCreate, db: Session = Depends(get_db)):
    db_item = Item(name=item.name, description=item.description or "")
    db.add(db_item)
    db.commit()
    db.refresh(db_item)
    return db_item


@router.get("/{item_id}", response_model=ItemResponse)
async def get_item(item_id: int, db: Session = Depends(get_db)):
    item = db.query(Item).filter(Item.id == item_id).first()
    if not item:
        raise HTTPException(status_code=404, detail="Item not found")
    return item
"#)?;

        std::fs::write(project_dir.join("app/routers/users.py"),
            r#"from fastapi import APIRouter

router = APIRouter()


@router.get("/")
async def list_users():
    return {"users": []}
"#)?;

        std::fs::write(project_dir.join("tests/__init__.py"), "")?;

        std::fs::write(project_dir.join("tests/test_items.py"),
            r#"from fastapi.testclient import TestClient
from main import app

client = TestClient(app)


def test_root():
    response = client.get("/")
    assert response.status_code == 200
    assert "message" in response.json()


def test_list_items():
    response = client.get("/api/items/")
    assert response.status_code == 200
"#)?;

        std::fs::write(project_dir.join(".env"),
            r#"APP_NAME=FastAPI App
DEBUG=True
DATABASE_URL=sqlite:///./app.db
"#)?;

        std::fs::write(project_dir.join(".env.example"),
            r#"APP_NAME=FastAPI App
DEBUG=True
DATABASE_URL=sqlite:///./app.db
"#)?;

        std::fs::write(project_dir.join("pytest.ini"),
            r#"[pytest]
testpaths = tests
asyncio_mode = auto
"#)?;

        std::fs::write(project_dir.join(".gitignore"),
            "*.pyc\n__pycache__\n.DS_Store\n*.db\n.env\n")?;

        std::fs::write(project_dir.join("README.md"),
            klyron_template::TemplateEngine::render_static(r#"# {{ name }}

FastAPI project

## Getting Started

pip install -r requirements.txt
uvicorn main:app --reload
"#, vars))?;

        Ok(())
    }
}
