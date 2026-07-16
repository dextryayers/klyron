import { readFileSync, statSync, writeFileSync, mkdirSync } from 'fs'
import { join } from 'path'

export function createAdapter() {
  return {
    name: 'fastapi',
    detect(dir) {
      try {
        const req = readFileSync(join(dir, 'requirements.txt'), 'utf-8').toLowerCase()
        if (req.includes('fastapi')) return true
      } catch {}
      try {
        const pyproject = readFileSync(join(dir, 'pyproject.toml'), 'utf-8').toLowerCase()
        if (pyproject.includes('fastapi')) return true
      } catch {}
      try {
        return statSync(join(dir, 'app', 'main.py')).isFile()
      } catch {}
      return false
    },
    supportedVersions: ['0.115'],
    defaultVersion: '0.115',
    kind: 'Polyglot',

    async dev(dir, port) {
      const { spawn } = await import('child_process')
      const proc = spawn('uvicorn', ['main:app', '--reload', '--host', '0.0.0.0', '--port', String(port || 8000)], { cwd: dir, stdio: 'inherit' })
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
      return scaffoldFastAPI(name, options)
    }
  }
}

function existsSync(p) {
  try { return statSync(p).isFile() } catch { return false }
}

async function scaffoldFastAPI(name, options) {
  const projectDir = join(options.dir || process.cwd(), name)

  mkdirSync(join(projectDir, 'app', 'routers'), { recursive: true })
  mkdirSync(join(projectDir, 'app', 'models'), { recursive: true })
  mkdirSync(join(projectDir, 'app', 'schemas'), { recursive: true })
  mkdirSync(join(projectDir, 'tests'), { recursive: true })

  writeFileSync(join(projectDir, 'requirements.txt'), `fastapi==0.115.6
uvicorn[standard]==0.34.0
sqlalchemy==2.0.36
psycopg2-binary==2.9.10
alembic==1.14.0
pydantic==2.10.3
pydantic-settings==2.7.0
python-dotenv==1.0.1
httpx==0.28.1
pytest==8.3.4
pytest-asyncio==0.25.0
ruff==0.8.4
`)

  writeFileSync(join(projectDir, '.env'), `DATABASE_URL=postgresql://postgres:postgres@localhost:5432/${name}
SECRET_KEY=change-me-to-a-random-secret-key
DEBUG=True
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

  writeFileSync(join(projectDir, 'app', '__init__.py'), ``)

  writeFileSync(join(projectDir, 'app', 'main.py'), `from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from app.routers import items, users
from app.database import engine, Base

Base.metadata.create_all(bind=engine)

app = FastAPI(
    title="${name}",
    version="1.0.0",
    description="${name} API built with FastAPI",
)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

app.include_router(items.router, prefix="/api/items", tags=["items"])
app.include_router(users.router, prefix="/api/users", tags=["users"])


@app.get("/health")
async def health():
    return {"status": "ok", "service": "${name}"}
`)

  writeFileSync(join(projectDir, 'app', 'database.py'), `from sqlalchemy import create_engine
from sqlalchemy.orm import DeclarativeBase, sessionmaker
from pydantic_settings import BaseSettings
from dotenv import load_dotenv

load_dotenv()


class Settings(BaseSettings):
    database_url: str = "sqlite:///./${name}.db"
    debug: bool = True

    model_config = {"env_file": ".env", "extra": "ignore"}


settings = Settings()

engine = create_engine(settings.database_url, echo=settings.debug)
SessionLocal = sessionmaker(autocommit=False, autoflush=False, bind=engine)


class Base(DeclarativeBase):
    pass


def get_db():
    db = SessionLocal()
    try:
        yield db
    finally:
        db.close()
`)

  writeFileSync(join(projectDir, 'app', 'routers', '__init__.py'), ``)

  writeFileSync(join(projectDir, 'app', 'routers', 'items.py'), `from fastapi import APIRouter, Depends, HTTPException, status
from sqlalchemy.orm import Session
from app.database import get_db
from app.models.item import Item
from app.schemas.item import ItemCreate, ItemRead, ItemUpdate

router = APIRouter()


@router.get("/", response_model=list[ItemRead])
async def list_items(skip: int = 0, limit: int = 100, db: Session = Depends(get_db)):
    items = db.query(Item).offset(skip).limit(limit).all()
    return items


@router.post("/", response_model=ItemRead, status_code=status.HTTP_201_CREATED)
async def create_item(payload: ItemCreate, db: Session = Depends(get_db)):
    item = Item(name=payload.name, description=payload.description, price=payload.price)
    db.add(item)
    db.commit()
    db.refresh(item)
    return item


@router.get("/{item_id}", response_model=ItemRead)
async def get_item(item_id: int, db: Session = Depends(get_db)):
    item = db.query(Item).filter(Item.id == item_id).first()
    if not item:
        raise HTTPException(status_code=404, detail="Item not found")
    return item


@router.patch("/{item_id}", response_model=ItemRead)
async def update_item(item_id: int, payload: ItemUpdate, db: Session = Depends(get_db)):
    item = db.query(Item).filter(Item.id == item_id).first()
    if not item:
        raise HTTPException(status_code=404, detail="Item not found")
    for key, value in payload.model_dump(exclude_unset=True).items():
        setattr(item, key, value)
    db.commit()
    db.refresh(item)
    return item


@router.delete("/{item_id}", status_code=status.HTTP_204_NO_CONTENT)
async def delete_item(item_id: int, db: Session = Depends(get_db)):
    item = db.query(Item).filter(Item.id == item_id).first()
    if not item:
        raise HTTPException(status_code=404, detail="Item not found")
    db.delete(item)
    db.commit()
`)

  writeFileSync(join(projectDir, 'app', 'routers', 'users.py'), `from fastapi import APIRouter, Depends, HTTPException, status
from sqlalchemy.orm import Session
from app.database import get_db
from app.models.user import User
from app.schemas.user import UserCreate, UserRead

router = APIRouter()


@router.get("/", response_model=list[UserRead])
async def list_users(skip: int = 0, limit: int = 100, db: Session = Depends(get_db)):
    users = db.query(User).offset(skip).limit(limit).all()
    return users


@router.post("/", response_model=UserRead, status_code=status.HTTP_201_CREATED)
async def create_user(payload: UserCreate, db: Session = Depends(get_db)):
    existing = db.query(User).filter(User.email == payload.email).first()
    if existing:
        raise HTTPException(status_code=409, detail="Email already registered")
    user = User(name=payload.name, email=payload.email)
    db.add(user)
    db.commit()
    db.refresh(user)
    return user


@router.get("/{user_id}", response_model=UserRead)
async def get_user(user_id: int, db: Session = Depends(get_db)):
    user = db.query(User).filter(User.id == user_id).first()
    if not user:
        raise HTTPException(status_code=404, detail="User not found")
    return user
`)

  writeFileSync(join(projectDir, 'app', 'models', '__init__.py'), `from app.models.item import Item
from app.models.user import User

__all__ = ["Item", "User"]
`)

  writeFileSync(join(projectDir, 'app', 'models', 'item.py'), `from sqlalchemy import Column, Integer, String, Float, Text
from app.database import Base


class Item(Base):
    __tablename__ = "items"

    id = Column(Integer, primary_key=True, index=True)
    name = Column(String(255), nullable=False)
    description = Column(Text, nullable=True)
    price = Column(Float, nullable=False)
`)

  writeFileSync(join(projectDir, 'app', 'models', 'user.py'), `from sqlalchemy import Column, Integer, String
from app.database import Base


class User(Base):
    __tablename__ = "users"

    id = Column(Integer, primary_key=True, index=True)
    name = Column(String(255), nullable=False)
    email = Column(String(255), unique=True, nullable=False, index=True)
`)

  writeFileSync(join(projectDir, 'app', 'schemas', '__init__.py'), ``)

  writeFileSync(join(projectDir, 'app', 'schemas', 'item.py'), `from pydantic import BaseModel
from typing import Optional


class ItemBase(BaseModel):
    name: str
    description: Optional[str] = None
    price: float


class ItemCreate(ItemBase):
    pass


class ItemUpdate(BaseModel):
    name: Optional[str] = None
    description: Optional[str] = None
    price: Optional[float] = None


class ItemRead(ItemBase):
    id: int

    model_config = {"from_attributes": True}
`)

  writeFileSync(join(projectDir, 'app', 'schemas', 'user.py'), `from pydantic import BaseModel, EmailStr
from typing import Optional


class UserBase(BaseModel):
    name: str
    email: EmailStr


class UserCreate(UserBase):
    pass


class UserRead(UserBase):
    id: int

    model_config = {"from_attributes": True}
`)

  writeFileSync(join(projectDir, 'tests', '__init__.py'), ``)

  writeFileSync(join(projectDir, 'tests', 'test_main.py'), `from fastapi.testclient import TestClient
from app.main import app

client = TestClient(app)


def test_health():
    resp = client.get("/health")
    assert resp.status_code == 200
    assert resp.json()["status"] == "ok"


def test_create_item():
    resp = client.post("/api/items/", json={"name": "Test", "description": "A test item", "price": 9.99})
    assert resp.status_code == 201
    data = resp.json()
    assert data["name"] == "Test"
    assert data["price"] == 9.99
`)

  writeFileSync(join(projectDir, 'README.md'), `# ${name}

FastAPI application

## Getting Started

\`\`\`bash
pip install -r requirements.txt
uvicorn app.main:app --reload --host 0.0.0.0 --port 8000
\`\`\`
`)
}
