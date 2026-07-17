#!/bin/bash
ADAPTERS="/home/aniippxploit/koding/klyron/adapters"

echo "=== BACKEND ==="

# --- Express ---
for ver in v4.18 v4.19 v4.21 v5.1; do
  dir="$ADAPTERS/backend/express/$ver"
  mkdir -p "$dir/src/routes" "$dir/src/middleware" "$dir/public"
  case $ver in
    v4.18) ev="^4.18.0"; jest="^29.0.0"; dep="" ;;
    v4.19) ev="^4.19.0"; jest="^29.0.0"; dep="" ;;
    v4.21) ev="^4.21.0"; jest="^30.0.0"; dep="" ;;
    v5.1) ev="^5.1.0"; jest="^30.0.0"; dep="", "cors": "^2.8.5" ;;
  esac
  cat > "$dir/package.json" << EOF
{ "name": "express-app", "version": "1.0.0", "private": true, "type": "module", "scripts": { "dev": "node --watch src/index.js", "start": "node src/index.js", "test": "jest" }, "dependencies": { "express": "$ev"$dep }, "devDependencies": { "jest": "$jest" } }
EOF
  cat > "$dir/src/index.js" << 'EOF'
import express from 'express'
const app = express()
const port = process.env.PORT || 3000
app.get('/', (req, res) => res.send('Hello Express'))
app.listen(port, () => console.log(`Server on :${port}`))
EOF
  cat > "$dir/src/routes/index.js" << 'EOF'
import { Router } from 'express'
const router = Router()
router.get('/', (req, res) => res.json({ message: 'ok' }))
export default router
EOF
  cat > "$dir/.gitignore" << 'EOF'
node_modules
.env
EOF
done

# --- Fastify ---
for ver in v4.0 v4.26 v4.28 v5.2; do
  dir="$ADAPTERS/backend/fastify/$ver"
  mkdir -p "$dir/src/routes" "$dir/src/plugins" "$dir/test"
  case $ver in
    v4.0) fv="^4.0.0" ;;
    v4.26) fv="^4.26.0" ;;
    v4.28) fv="^4.28.0" ;;
    v5.2) fv="^5.2.0" ;;
  esac
  cat > "$dir/package.json" << EOF
{ "name": "fastify-app", "version": "1.0.0", "private": true, "type": "module", "scripts": { "dev": "node --watch src/index.js", "start": "node src/index.js" }, "dependencies": { "fastify": "$fv" } }
EOF
  cat > "$dir/src/index.js" << 'EOF'
import Fastify from 'fastify'
const app = Fastify()
app.get('/', async () => ({ hello: 'Fastify' }))
await app.listen({ port: 3000 })
EOF
done

# --- NestJS ---
for ver in v9.0 v10.0 v10.3 v11.0; do
  dir="$ADAPTERS/backend/nestjs/$ver"
  mkdir -p "$dir/src" "$dir/test"
  case $ver in
    v9.0) nv="^9.0.0"; tv="^5.0.0" ;;
    v10.0) nv="^10.0.0"; tv="^5.0.0" ;;
    v10.3) nv="^10.3.0"; tv="^5.0.0" ;;
    v11.0) nv="^11.0.0"; tv="^6.0.0" ;;
  esac
  cat > "$dir/package.json" << EOF
{ "name": "nest-app", "version": "1.0.0", "private": true, "scripts": { "start": "nest start", "dev": "nest start --watch" }, "dependencies": { "@nestjs/core": "$nv", "@nestjs/common": "$nv", "@nestjs/platform-express": "$nv", "reflect-metadata": "^0.2.0", "rxjs": "^7.8.0" }, "devDependencies": { "@nestjs/cli": "$nv", "@nestjs/testing": "$nv", "typescript": "^5.4.0" } }
EOF
  cat > "$dir/src/main.ts" << 'EOF'
import { NestFactory } from '@nestjs/core'
import { AppModule } from './app.module'
async function bootstrap() { const app = await NestFactory.create(AppModule); await app.listen(3000) }
bootstrap()
EOF
  cat > "$dir/src/app.module.ts" << 'EOF'
import { Module } from '@nestjs/common'
import { AppController } from './app.controller'
@Module({ controllers: [AppController] })
export class AppModule {}
EOF
  cat > "$dir/src/app.controller.ts" << 'EOF'
import { Controller, Get } from '@nestjs/common'
@Controller()
export class AppController { @Get() getHello() { return 'Hello NestJS' } }
EOF
  cat > "$dir/tsconfig.json" << 'EOF'
{ "compilerOptions": { "module": "commonjs", "target": "ES2022", "strict": true } }
EOF
  cat > "$dir/nest-cli.json" << 'EOF'
{ "collection": "@nestjs/schematics", "sourceRoot": "src" }
EOF
done

# --- Hono ---
for ver in v3.0 v3.12 v4.0 v4.7; do
  dir="$ADAPTERS/backend/hono/$ver"
  mkdir -p "$dir/src/routes" "$dir/src/middleware"
  case $ver in
    v3.0) hv="^3.0.0" ;;
    v3.12) hv="^3.12.0" ;;
    v4.0) hv="^4.0.0" ;;
    v4.7) hv="^4.7.0" ;;
  esac
  cat > "$dir/package.json" << EOF
{ "name": "hono-app", "version": "1.0.0", "private": true, "type": "module", "scripts": { "dev": "tsx watch src/index.ts" }, "dependencies": { "hono": "$hv" }, "devDependencies": { "tsx": "^4.0.0", "typescript": "^5.7.0" } }
EOF
  cat > "$dir/src/index.ts" << 'EOF'
import { Hono } from 'hono'
const app = new Hono()
app.get('/', (c) => c.text('Hello Hono'))
export default app
EOF
done

# --- Elysia ---
for ver in v0.1 v0.8 v1.0 v1.2; do
  dir="$ADAPTERS/backend/elysia/$ver"
  case $ver in
    v0.1) ev="^0.1.0" ;;
    v0.8) ev="^0.8.0" ;;
    v1.0) ev="^1.0.0" ;;
    v1.2) ev="^1.2.0" ;;
  esac
  cat > "$dir/package.json" << EOF
{ "name": "elysia-app", "version": "1.0.0", "private": true, "type": "module", "scripts": { "dev": "bun run src/index.ts" }, "dependencies": { "elysia": "$ev" } }
EOF
  mkdir -p "$dir/src"
  cat > "$dir/src/index.ts" << 'EOF'
import { Elysia } from 'elysia'
new Elysia().get('/', () => 'Hello Elysia').listen(3000)
EOF
done

# --- Koa ---
for ver in v2.0 v2.13 v2.14 v2.15; do
  dir="$ADAPTERS/backend/koa/$ver"
  mkdir -p "$dir/src/routes" "$dir/src/middleware"
  cat > "$dir/package.json" << EOF
{ "name": "koa-app", "version": "1.0.0", "private": true, "type": "module", "scripts": { "dev": "node --watch src/index.js" }, "dependencies": { "koa": "^2.15.0" } }
EOF
  cat > "$dir/src/index.js" << 'EOF'
import Koa from 'koa'
const app = new Koa()
app.use(ctx => ctx.body = 'Hello Koa')
app.listen(3000)
EOF
done

# --- Hapi ---
for ver in v18.0 v19.0 v20.0 v21.0; do
  dir="$ADAPTERS/backend/hapi/$ver"
  mkdir -p "$dir/src/routes" "$dir/src/plugins"
  case $ver in
    v18.0) hv="^18.0.0" ;;
    v19.0) hv="^19.0.0" ;;
    v20.0) hv="^20.0.0" ;;
    v21.0) hv="^21.0.0" ;;
  esac
  cat > "$dir/package.json" << EOF
{ "name": "hapi-app", "version": "1.0.0", "private": true, "type": "module", "scripts": { "dev": "node --watch src/index.js" }, "dependencies": { "@hapi/hapi": "$hv" } }
EOF
  cat > "$dir/src/index.js" << 'EOF'
import Hapi from '@hapi/hapi'
const init = async () => { const server = Hapi.server({ port: 3000 }); server.route({ method: 'GET', path: '/', handler: () => 'Hello Hapi' }); await server.start() }
init()
EOF
done

# --- AdonisJS ---
for ver in v5.0 v6.0 v6.15 v7.0; do
  dir="$ADAPTERS/backend/adonis/$ver"
  mkdir -p "$dir/start" "$dir/config" "$dir/app/Controllers/Http"
  case $ver in
    v5.0) av="^5.0.0"; cli="^5.0.0" ;;
    v6.0) av="^6.0.0"; cli="^6.0.0" ;;
    v6.15) av="^6.15.0"; cli="^6.0.0" ;;
    v7.0) av="^7.0.0"; cli="^7.0.0" ;;
  esac
  cat > "$dir/package.json" << EOF
{ "name": "adonis-app", "version": "1.0.0", "private": true, "scripts": { "dev": "node ace serve --watch" }, "dependencies": { "@adonisjs/core": "$av" }, "devDependencies": { "typescript": "^5.4.0" } }
EOF
  cat > "$dir/start/routes.ts" << 'EOF'
import Route from '@ioc:Adonis/Core/Route'
Route.get('/', async () => 'Hello Adonis')
EOF
done

# --- Fastest way for remaining backend frameworks ---
for fw in feathers loopback moleculer nitro polka restify socket.io actionhero totaljs foalts tsed midwayjs overnightjs nestia; do
  for ver in $(ls "$ADAPTERS/backend/$fw/" 2>/dev/null); do
    dir="$ADAPTERS/backend/$fw/$ver"
    cat > "$dir/package.json" << EOF
{ "name": "$fw-app", "version": "1.0.0", "private": true, "scripts": { "dev": "node index.js" }, "dependencies": { "$fw": "^1.0.0" } }
EOF
    cat > "$dir/index.js" << 'EOF'
console.log('Hello from ' + process.env.npm_package_name)
EOF
  done
done

echo "=== BACKEND DONE ==="
