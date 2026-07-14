use std::path::Path;
use async_trait::async_trait;
use anyhow::Result;
use crate::{FrameworkAdapter, BuildOptions, ScaffoldOptions, FrameworkKind};

pub struct NestJsAdapter;

#[async_trait]
impl FrameworkAdapter for NestJsAdapter {
    fn name(&self) -> &'static str { "nestjs" }

    fn detect(&self, dir: &Path) -> bool {
        dir.join("nest-cli.json").exists() || dir.join("nest.config.json").exists()
    }

    fn supported_versions(&self) -> Vec<&'static str> { vec!["10.0"] }
    fn default_version(&self) -> &'static str { "10.0" }
    fn kind(&self) -> FrameworkKind { FrameworkKind::Backend }

    async fn dev(&self, dir: &Path, port: Option<u16>) -> Result<()> {
        let mut cmd = tokio::process::Command::new("npx");
        cmd.args(["nest", "start", "--watch"]).current_dir(dir);
        if let Some(p) = port { cmd.env("PORT", p.to_string()); }
        cmd.status().await?; Ok(())
    }

    async fn build(&self, dir: &Path, _opts: BuildOptions) -> Result<()> {
        tokio::process::Command::new("npx").args(["nest", "build"]).current_dir(dir).status().await?;
        Ok(())
    }

    async fn test(&self, dir: &Path, _filter: Option<&str>) -> Result<()> {
        tokio::process::Command::new("npx").args(["jest"]).current_dir(dir).status().await?;
        Ok(())
    }

    async fn lint(&self, dir: &Path, _fix: bool) -> Result<()> {
        tokio::process::Command::new("npx").args(["eslint", "."]).current_dir(dir).status().await?;
        Ok(())
    }

    async fn format(&self, dir: &Path, write: bool) -> Result<()> {
        tokio::process::Command::new("npx")
            .args(if write { ["prettier", "--write", "."] } else { ["prettier", "--check", "."] })
            .current_dir(dir).status().await?;
        Ok(())
    }

    fn external_scaffold_command(&self, name: &str, _version: Option<&str>) -> Option<(String, Vec<String>)> {
        Some(("npx".into(), vec!["@nestjs/cli@latest".into(), "new".into(), name.into()]))
    }

    async fn scaffold(&self, name: &str, options: ScaffoldOptions) -> Result<()> {
        if options.external {
            if let Some((cmd, args)) = self.external_scaffold_command(name, options.version.as_deref()) {
                let status = std::process::Command::new(&cmd).args(&args).current_dir(&options.dir).status()?;
                if !status.success() { anyhow::bail!("External scaffolding failed"); }
                return Ok(());
            }
        }
        let project_dir = options.dir.join(name);
        std::fs::create_dir_all(&project_dir)?;
        std::fs::create_dir_all(project_dir.join("src"))?;
        std::fs::create_dir_all(project_dir.join("src/controllers"))?;
        std::fs::create_dir_all(project_dir.join("src/services"))?;
        std::fs::create_dir_all(project_dir.join("src/modules"))?;
        std::fs::create_dir_all(project_dir.join("test"))?;

        let vars = &options.template_vars;

        std::fs::write(project_dir.join("nest-cli.json"),
            r#"{
  "$schema": "https://json.schemastore.org/nest-cli",
  "collection": "@nestjs/schematics",
  "sourceRoot": "src",
  "compilerOptions": { "deleteOutDir": true }
}"#)?;

        std::fs::write(project_dir.join("package.json"),
            klyron_template::TemplateEngine::render_static(r#"{
  "name": "{{ name }}",
  "version": "1.0.0",
  "private": true,
  "scripts": {
    "build": "nest build",
    "dev": "nest start --watch",
    "start": "node dist/main",
    "test": "jest",
    "lint": "eslint .",
    "format": "prettier --write ."
  },
  "dependencies": {
    "@nestjs/common": "^10.0.0",
    "@nestjs/core": "^10.0.0",
    "@nestjs/platform-express": "^10.0.0",
    "reflect-metadata": "^0.2.0",
    "rxjs": "^7.8.0"
  },
  "devDependencies": {
    "@nestjs/cli": "^10.0.0",
    "@nestjs/schematics": "^10.0.0",
    "@nestjs/testing": "^10.0.0",
    "typescript": "^5.6.0",
    "ts-node": "^10.9.0",
    "jest": "^29.7.0",
    "eslint": "^9.0.0",
    "@nestjs/eslint-config": "^10.0.0",
    "prettier": "^3.4.0"
  }
}"#, vars))?;

        std::fs::write(project_dir.join("tsconfig.json"),
            r#"{
  "compilerOptions": {
    "module": "commonjs",
    "declaration": true,
    "removeComments": true,
    "emitDecoratorMetadata": true,
    "experimentalDecorators": true,
    "allowSyntheticDefaultImports": true,
    "target": "ES2021",
    "sourceMap": true,
    "outDir": "./dist",
    "baseUrl": "./",
    "incremental": true,
    "skipLibCheck": true,
    "strictNullChecks": true,
    "noImplicitAny": true,
    "strictBindCallApply": true,
    "forceConsistentCasingInFileNames": true,
    "noFallthroughCasesInSwitch": true
  }
}"#)?;

        std::fs::write(project_dir.join("tsconfig.build.json"),
            r#"{
  "extends": "./tsconfig.json",
  "exclude": ["node_modules", "test", "dist", "**/*spec.ts"]
}"#)?;

        std::fs::write(project_dir.join("src/main.ts"),
            klyron_template::TemplateEngine::render_static(r#"import { NestFactory } from '@nestjs/core'
import { AppModule } from './app.module'

async function bootstrap() {
  const app = await NestFactory.create(AppModule)
  await app.listen(3000)
  console.log(`{{ name }} running on http://localhost:3000`)
}
bootstrap()
"#, vars))?;

        std::fs::write(project_dir.join("src/app.module.ts"),
            r#"import { Module } from '@nestjs/common'
import { AppController } from './app.controller'
import { AppService } from './app.service'

@Module({
  imports: [],
  controllers: [AppController],
  providers: [AppService],
})
export class AppModule {}
"#)?;

        std::fs::write(project_dir.join("src/app.controller.ts"),
            r#"import { Controller, Get } from '@nestjs/common'
import { AppService } from './app.service'

@Controller()
export class AppController {
  constructor(private readonly appService: AppService) {}

  @Get()
  getHello(): string {
    return this.appService.getHello()
  }
}
"#)?;

        std::fs::write(project_dir.join("src/app.service.ts"),
            r#"import { Injectable } from '@nestjs/common'

@Injectable()
export class AppService {
  getHello(): string {
    return 'Hello World!'
  }
}
"#)?;

        std::fs::write(project_dir.join("test/app.e2e-spec.ts"),
            r#"import { Test, TestingModule } from '@nestjs/testing'
import { INestApplication } from '@nestjs/common'
import * as request from 'supertest'
import { AppModule } from '../src/app.module'

describe('AppController (e2e)', () => {
  let app: INestApplication

  beforeEach(async () => {
    const moduleFixture: TestingModule = await Test.createTestingModule({
      imports: [AppModule],
    }).compile()

    app = moduleFixture.createNestApplication()
    await app.init()
  })

  it('/ (GET)', () => {
    return request(app.getHttpServer()).get('/').expect(200).expect('Hello World!')
  })
})
"#)?;

        std::fs::write(project_dir.join("src/app.controller.spec.ts"),
            r#"import { Test, TestingModule } from '@nestjs/testing'
import { AppController } from './app.controller'
import { AppService } from './app.service'

describe('AppController', () => {
  let appController: AppController

  beforeEach(async () => {
    const app: TestingModule = await Test.createTestingModule({
      controllers: [AppController],
      providers: [AppService],
    }).compile()

    appController = app.get<AppController>(AppController)
  })

  describe('root', () => {
    it('should return "Hello World!"', () => {
      expect(appController.getHello()).toBe('Hello World!')
    })
  })
})
"#)?;

        std::fs::write(project_dir.join(".gitignore"), "node_modules\ndist\n.DS_Store\n*.log\n")?;
        std::fs::write(project_dir.join(".prettierrc"),
            r#"{"semi": true, "singleQuote": true, "tabWidth": 2, "trailingComma": "es5", "printWidth": 100}"#)?;
        std::fs::write(project_dir.join("eslint.config.js"),
            r#"import nest from '@nestjs/eslint-config'
export default [...nest.configs.recommended, { ignores: ['dist'] }]"#)?;
        std::fs::write(project_dir.join("README.md"),
            klyron_template::TemplateEngine::render_static(r#"# {{ name }}

NestJS project

## Getting Started

npm run dev
"#, vars))?;

        Ok(())
    }
}
