use clap::Subcommand;

#[derive(Subcommand)]
pub enum DockerAction {
    Init,
    Build,
    Run,
}

pub fn run_docker(action: DockerAction) -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    match action {
        DockerAction::Init => docker_init(&dir),
        DockerAction::Build => {
            crate::run_cmd("docker", &["build", "-t", "klyron-app", "."], &dir)
        }
        DockerAction::Run => {
            crate::run_cmd("docker", &["run", "-p", "3000:3000", "klyron-app"], &dir)
        }
    }
}

fn docker_init(dir: &std::path::Path) -> anyhow::Result<()> {
    let project = crate::detect_project_type(dir);
    let dockerfile = match project {
        "node" => r#"FROM node:22-alpine AS base
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
EXPOSE 3000
CMD ["npm", "start"]
"#,
        "laravel" => r#"FROM php:8.3-fpm AS base
RUN apt-get update && apt-get install -y nginx
COPY --from=composer:latest /usr/bin/composer /usr/bin/composer
WORKDIR /var/www
COPY composer*.json ./
RUN composer install --no-dev
COPY . .
EXPOSE 80
CMD ["php", "artisan", "serve", "--host=0.0.0.0", "--port=80"]
"#,
        "rust" => r#"FROM rust:latest AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/ ./
EXPOSE 3000
CMD ["./app"]
"#,
        "python" => r#"FROM python:3.12-slim
WORKDIR /app
COPY requirements.txt ./
RUN pip install -r requirements.txt
COPY . .
EXPOSE 8000
CMD ["python3", "manage.py", "runserver", "0.0.0.0:8000"]
"#,
        "go" => r#"FROM golang:latest AS builder
WORKDIR /app
COPY go.* ./
RUN go mod download
COPY . .
RUN go build -o app .
FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/app ./
EXPOSE 3000
CMD ["./app"]
"#,
        _ => r#"FROM node:22-alpine
WORKDIR /app
COPY . .
EXPOSE 3000
CMD ["npm", "start"]
"#,
    };
    std::fs::write(dir.join("Dockerfile"), dockerfile)?;
    let compose = r#"version: '3.8'
services:
  app:
    build: .
    ports:
      - "3000:3000"
    volumes:
      - .:/app
    environment:
      - NODE_ENV=development
"#;
    std::fs::write(dir.join("docker-compose.yml"), compose)?;
    std::fs::write(dir.join(".dockerignore"), "node_modules\ntarget\n.git\n*.md\n")?;
    println!("✅ Docker files generated:");
    println!("  Dockerfile, docker-compose.yml, .dockerignore");
    Ok(())
}
