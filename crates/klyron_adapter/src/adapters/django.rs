use std::collections::HashMap;
use std::path::Path;
use async_trait::async_trait;
use anyhow::Result;
use crate::{FrameworkAdapter, BuildOptions, ScaffoldOptions, FrameworkKind};

pub struct DjangoAdapter;

#[async_trait]
impl FrameworkAdapter for DjangoAdapter {
    fn name(&self) -> &'static str { "django" }

    fn detect(&self, dir: &Path) -> bool {
        dir.join("manage.py").exists()
    }

    fn supported_versions(&self) -> Vec<&'static str> { vec!["4.2", "5.0"] }
    fn default_version(&self) -> &'static str { "5.0" }
    fn kind(&self) -> FrameworkKind { FrameworkKind::Polyglot }

    async fn dev(&self, dir: &Path, port: Option<u16>) -> Result<()> {
        let mut cmd = tokio::process::Command::new("python3");
        cmd.args(["manage.py", "runserver"]).current_dir(dir);
        if let Some(p) = port { cmd.arg(format!("0.0.0.0:{}", p)); }
        cmd.status().await?; Ok(())
    }

    async fn build(&self, _dir: &Path, _opts: BuildOptions) -> Result<()> { Ok(()) }

    async fn test(&self, dir: &Path, _filter: Option<&str>) -> Result<()> {
        tokio::process::Command::new("python3").args(["-m", "pytest", "."]).current_dir(dir).status().await?;
        Ok(())
    }

    async fn lint(&self, dir: &Path, _fix: bool) -> Result<()> {
        tokio::process::Command::new("ruff").args(["check", "."]).current_dir(dir).status().await?;
        Ok(())
    }

    async fn format(&self, dir: &Path, write: bool) -> Result<()> {
        if write {
            tokio::process::Command::new("black").args(["."]).current_dir(dir).status().await?;
        } else {
            tokio::process::Command::new("black").args(["--check", "."]).current_dir(dir).status().await?;
        }
        Ok(())
    }

    async fn scaffold(&self, name: &str, options: ScaffoldOptions) -> Result<()> {
        let project_dir = options.dir.join(name);
        std::fs::create_dir_all(&project_dir)?;
        std::fs::create_dir_all(project_dir.join("config"))?;
        std::fs::create_dir_all(project_dir.join("apps/core"))?;
        std::fs::create_dir_all(project_dir.join("apps/users"))?;
        std::fs::create_dir_all(project_dir.join("templates"))?;
        std::fs::create_dir_all(project_dir.join("static/css"))?;
        std::fs::create_dir_all(project_dir.join("media"))?;
        std::fs::create_dir_all(project_dir.join("tests"))?;

        let vars = &options.template_vars;

        std::fs::write(project_dir.join("manage.py"),
            klyron_template::TemplateEngine::render(r#"#!/usr/bin/env python3
import os, sys
def main():
    os.environ.setdefault('DJANGO_SETTINGS_MODULE', 'config.settings')
    from django.core.management import execute_from_command_line
    execute_from_command_line(sys.argv)
if __name__ == '__main__':
    main()
"#, vars))?;

        std::fs::write(project_dir.join("requirements.txt"),
            r#"django>=5.0,<5.1
djangorestframework>=3.15,<4.0
django-cors-headers>=4.0,<5.0
celery>=5.4,<6.0
redis>=5.0,<6.0
psycopg2-binary>=2.9,<3.0
gunicorn>=22.0,<23.0
pytest-django>=4.8,<5.0
ruff>=0.6,<1.0
black>=24.0,<25.0
"#)?;

        std::fs::write(project_dir.join("config/__init__.py"), "")?;

        std::fs::write(project_dir.join("config/settings.py"),
            klyron_template::TemplateEngine::render(r#"import os
from pathlib import Path

BASE_DIR = Path(__file__).resolve().parent.parent
SECRET_KEY = 'django-insecure-change-me'
DEBUG = True
ALLOWED_HOSTS = ['*']

INSTALLED_APPS = [
    'django.contrib.admin',
    'django.contrib.auth',
    'django.contrib.contenttypes',
    'django.contrib.sessions',
    'django.contrib.messages',
    'django.contrib.staticfiles',
    'rest_framework',
    'corsheaders',
    'apps.core',
    'apps.users',
]

MIDDLEWARE = [
    'corsheaders.middleware.CorsMiddleware',
    'django.middleware.security.SecurityMiddleware',
    'django.contrib.sessions.middleware.SessionMiddleware',
    'django.middleware.common.CommonMiddleware',
    'django.middleware.csrf.CsrfViewMiddleware',
    'django.contrib.auth.middleware.AuthenticationMiddleware',
    'django.contrib.messages.middleware.MessageMiddleware',
    'django.middleware.clickjacking.XFrameOptionsMiddleware',
]

ROOT_URLCONF = 'config.urls'
TEMPLATES = [{'BACKEND': 'django.template.backends.django.DjangoTemplates', 'DIRS': [BASE_DIR / 'templates'], 'APP_DIRS': True}]
WSGI_APPLICATION = 'config.wsgi.application'
DATABASES = {'default': {'ENGINE': 'django.db.backends.sqlite3', 'NAME': BASE_DIR / 'db.sqlite3'}}
AUTH_PASSWORD_VALIDATORS = []
LANGUAGE_CODE = 'en-us'
TIME_ZONE = 'UTC'
USE_I18N = True
USE_TZ = True
STATIC_URL = 'static/'
STATICFILES_DIRS = [BASE_DIR / 'static']
MEDIA_URL = 'media/'
MEDIA_ROOT = BASE_DIR / 'media'
DEFAULT_AUTO_FIELD = 'django.db.models.BigAutoField'
CORS_ALLOW_ALL_ORIGINS = True
"#, vars))?;

        std::fs::write(project_dir.join("config/urls.py"),
            r#"from django.contrib import admin
from django.urls import path, include

urlpatterns = [
    path('admin/', admin.site.urls),
    path('api/', include('apps.core.urls')),
    path('api/users/', include('apps.users.urls')),
]
"#)?;

        std::fs::write(project_dir.join("config/wsgi.py"),
            r#"import os
from django.core.wsgi import get_wsgi_application
os.environ.setdefault('DJANGO_SETTINGS_MODULE', 'config.settings')
application = get_wsgi_application()
"#)?;

        std::fs::write(project_dir.join("config/asgi.py"),
            r#"import os
from django.core.asgi import get_asgi_application
os.environ.setdefault('DJANGO_SETTINGS_MODULE', 'config.settings')
application = get_asgi_application()
"#)?;

        std::fs::write(project_dir.join("apps/core/__init__.py"), "")?;
        std::fs::write(project_dir.join("apps/core/models.py"),
            r#"from django.db import models

class Item(models.Model):
    name = models.CharField(max_length=255)
    description = models.TextField(blank=True)
    created_at = models.DateTimeField(auto_now_add=True)

    def __str__(self):
        return self.name
"#)?;

        std::fs::write(project_dir.join("apps/core/views.py"),
            r#"from rest_framework import viewsets
from .models import Item
from .serializers import ItemSerializer

class ItemViewSet(viewsets.ModelViewSet):
    queryset = Item.objects.all()
    serializer_class = ItemSerializer
"#)?;

        std::fs::write(project_dir.join("apps/core/urls.py"),
            r#"from django.urls import path, include
from rest_framework.routers import DefaultRouter
from . import views

router = DefaultRouter()
router.register('items', views.ItemViewSet)

urlpatterns = [path('', include(router.urls))]
"#)?;

        std::fs::write(project_dir.join("apps/core/admin.py"),
            r#"from django.contrib import admin
from .models import Item

@admin.register(Item)
class ItemAdmin(admin.ModelAdmin):
    list_display = ['name', 'created_at']
"#)?;

        std::fs::write(project_dir.join("apps/core/apps.py"),
            r#"from django.apps import AppConfig

class CoreConfig(AppConfig):
    default_auto_field = 'django.db.models.BigAutoField'
    name = 'apps.core'
"#)?;

        std::fs::write(project_dir.join("apps/core/serializers.py"),
            r#"from rest_framework import serializers
from .models import Item

class ItemSerializer(serializers.ModelSerializer):
    class Meta:
        model = Item
        fields = '__all__'
"#)?;

        std::fs::write(project_dir.join("apps/core/tests.py"),
            r#"from django.test import TestCase
from .models import Item

class ItemModelTest(TestCase):
    def test_create_item(self):
        item = Item.objects.create(name='Test')
        self.assertEqual(str(item), 'Test')
"#)?;

        std::fs::write(project_dir.join("apps/users/__init__.py"), "")?;
        std::fs::write(project_dir.join("apps/users/models.py"),
            r#"from django.contrib.auth.models import AbstractUser

class User(AbstractUser):
    pass
"#)?;

        std::fs::write(project_dir.join("apps/users/views.py"),
            r#"from rest_framework import viewsets
from .models import User
from .serializers import UserSerializer

class UserViewSet(viewsets.ModelViewSet):
    queryset = User.objects.all()
    serializer_class = UserSerializer
"#)?;

        std::fs::write(project_dir.join("apps/users/urls.py"),
            r#"from django.urls import path, include
from rest_framework.routers import DefaultRouter
from . import views

router = DefaultRouter()
router.register('', views.UserViewSet)

urlpatterns = [path('', include(router.urls))]
"#)?;

        std::fs::write(project_dir.join("apps/users/admin.py"),
            r#"from django.contrib import admin
from django.contrib.auth.admin import UserAdmin
from .models import User

admin.site.register(User, UserAdmin)
"#)?;

        std::fs::write(project_dir.join("apps/users/serializers.py"),
            r#"from rest_framework import serializers
from .models import User

class UserSerializer(serializers.ModelSerializer):
    class Meta:
        model = User
        fields = ['id', 'username', 'email']
"#)?;

        std::fs::write(project_dir.join("templates/base.html"),
            klyron_template::TemplateEngine::render(r#"<!doctype html>
<html><head><meta charset="utf-8"><title>{% block title %}{{ name }}{% endblock %}</title></head>
<body>{% block content %}{% endblock %}</body></html>
"#, vars))?;

        std::fs::write(project_dir.join("static/css/style.css"),
            r#"* { margin: 0; padding: 0; box-sizing: border-box; }
body { font-family: system-ui, sans-serif; }
"#)?;

        std::fs::write(project_dir.join(".env.example"),
            r#"DJANGO_SECRET_KEY=change-me
DJANGO_DEBUG=True
DATABASE_URL=sqlite:///db.sqlite3
"#)?;

        std::fs::write(project_dir.join("pytest.ini"),
            r#"[pytest]
DJANGO_SETTINGS_MODULE = config.settings
python_files = tests.py test_*.py
"#)?;

        std::fs::write(project_dir.join(".gitignore"),
            "node_modules\n*.pyc\n__pycache__\n.DS_Store\n*.sqlite3\nmedia\n")?;

        std::fs::write(project_dir.join("README.md"),
            klyron_template::TemplateEngine::render(r#"# {{ name }}

Django project

## Getting Started

pip install -r requirements.txt
python manage.py migrate
python manage.py runserver
"#, vars))?;

        Ok(())
    }
}
