#!/bin/bash
set -e
ADAPTERS="/home/aniippxploit/koding/klyron/adapters"

generate_package_json() {
  local dir=$1 name=$2 version=$3 deps=$4 devdeps=$5
  mkdir -p "$dir"
  cat > "$dir/package.json" << EOF
{
  "name": "$name",
  "version": "1.0.0",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview"
  },
  "dependencies": {$deps},
  "devDependencies": {$devdeps}
}
EOF
}

generate_vite_config() {
  local dir=$1 plugin=$2
  cat > "$dir/vite.config.ts" << EOF
import { defineConfig } from 'vite'
import $plugin

export default defineConfig({
  plugins: [$plugin()],
})
EOF
}

# ================================================================
# FRONTEND
# ================================================================

echo "=== FRONTEND ==="

# --- React ---
REACT_DEPS='"react": "^19.1.0", "react-dom": "^19.1.0"'
REACT_DEVDEPS='"@vitejs/plugin-react": "^4.4.0", "typescript": "^5.7.0", "vite": "^6.2.0", "@types/react": "^19.1.0", "@types/react-dom": "^19.1.0"'
for ver in v18.2 v18.3 v19.0 v19.1; do
  dir="$ADAPTERS/frontend/react/$ver"
  case $ver in
    v18.2) deps='"react": "^18.2.0", "react-dom": "^18.2.0"'; atypes='"@types/react": "^18.2.0", "@types/react-dom": "^18.2.0"' ;;
    v18.3) deps='"react": "^18.3.0", "react-dom": "^18.3.0"'; atypes='"@types/react": "^18.3.0", "@types/react-dom": "^18.3.0"' ;;
    v19.0) deps='"react": "^19.0.0", "react-dom": "^19.0.0"'; atypes='"@types/react": "^19.0.0", "@types/react-dom": "^19.0.0"' ;;
    v19.1) deps=$REACT_DEPS; atypes='"@types/react": "^19.1.0", "@types/react-dom": "^19.1.0"' ;;
  esac
  generate_package_json "$dir" "react-app" "$ver" "$deps" '"@vitejs/plugin-react": "^4.4.0", "typescript": "^5.7.0", "vite": "^6.2.0", '"$atypes"
  cat > "$dir/index.html" << EOF
<!DOCTYPE html><html lang="en"><head><meta charset="UTF-8" /><meta name="viewport" content="width=device-width, initial-scale=1.0" /><title>React App</title></head><body><div id="root"></div><script type="module" src="/src/main.tsx"></script></body></html>
EOF
  mkdir -p "$dir/src"
  cat > "$dir/src/main.tsx" << 'EOF'
import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import App from './App'
createRoot(document.getElementById('root')!).render(<StrictMode><App /></StrictMode>)
EOF
  cat > "$dir/src/App.tsx" << 'EOF'
export default function App() { return <h1>Hello React</h1> }
EOF
  cat > "$dir/src/vite-env.d.ts" << 'EOF'
/// <reference types="vite/client" />
EOF
  cat > "$dir/tsconfig.json" << 'EOF'
{ "compilerOptions": { "target": "ES2022", "jsx": "react-jsx", "module": "ESNext", "moduleResolution": "bundler", "strict": true, "noEmit": true }, "include": ["src"] }
EOF
  generate_vite_config "$dir" "react from '@vitejs/plugin-react'"
done

# --- Next.js ---
for ver in v14 v15.0 v15.1 v15.2; do
  dir="$ADAPTERS/frontend/next/$ver"
  mkdir -p "$dir/app" "$dir/public" "$dir/styles"
  case $ver in
    v14) nv="^14.0.0"; ec="^14.0.0"; tw="^3.4.0" ;;
    v15.0) nv="^15.0.0"; ec="^15.0.0"; tw="^4.0.0" ;;
    v15.1) nv="^15.1.0"; ec="^15.1.0"; tw="^4.0.0" ;;
    v15.2) nv="^15.2.0"; ec="^15.2.0"; tw="^4.0.0" ;;
  esac
  cat > "$dir/package.json" << EOF
{ "name": "next-app", "version": "1.0.0", "private": true, "scripts": { "dev": "next dev", "build": "next build", "start": "next start" }, "dependencies": { "next": "$nv", "react": "^19.1.0", "react-dom": "^19.1.0" }, "devDependencies": { "typescript": "^5.7.0", "@types/node": "^22.0.0", "@types/react": "^19.1.0", "@types/react-dom": "^19.1.0", "tailwindcss": "$tw", "eslint": "^9.0.0", "eslint-config-next": "$ec" } }
EOF
  cat > "$dir/tsconfig.json" << 'EOF'
{ "compilerOptions": { "target": "ES2022", "lib": ["dom","dom.iterable","esnext"], "allowJs": true, "skipLibCheck": true, "strict": true, "noEmit": true, "module": "esnext", "moduleResolution": "bundler", "jsx": "preserve", "plugins": [{ "name": "next" }] }, "include": ["next-env.d.ts","**/*.ts","**/*.tsx"] }
EOF
  cat > "$dir/app/layout.tsx" << 'EOF'
export default function RootLayout({ children }: { children: React.ReactNode }) { return <html lang="en"><body>{children}</body></html> }
EOF
  cat > "$dir/app/page.tsx" << 'EOF'
export default function Home() { return <h1>Hello Next.js</h1> }
EOF
  cat > "$dir/next.config.ts" << 'EOF'
import type { NextConfig } from 'next'
const nextConfig: NextConfig = {}
export default nextConfig
EOF
  cat > "$dir/.gitignore" << 'EOF'
node_modules
.next
*.tsbuildinfo
EOF
  cat > "$dir/next-env.d.ts" << 'EOF'
/// <reference types="next" />
/// <reference types="next/image-types/global" />
EOF
  if [ "$nv" = "^14.0.0" ]; then
    cat > "$dir/next.config.js" << 'EOF'
/** @type {import('next').NextConfig} */
const nextConfig = {}
module.exports = nextConfig
EOF
    rm -f "$dir/next.config.ts"
  fi
done

# --- Vue ---
for ver in v3.4 v3.5 v3.6 v3.7; do
  dir="$ADAPTERS/frontend/vue/$ver"
  case $ver in
    v3.4) vv="^3.4.0" ;;
    v3.5) vv="^3.5.0" ;;
    v3.6) vv="^3.6.0" ;;
    v3.7) vv="^3.7.0" ;;
  esac
  mkdir -p "$dir/src"
  cat > "$dir/package.json" << EOF
{ "name": "vue-app", "version": "1.0.0", "private": true, "scripts": { "dev": "vite", "build": "vite build" }, "dependencies": { "vue": "$vv" }, "devDependencies": { "@vitejs/plugin-vue": "^5.2.0", "vite": "^6.2.0", "typescript": "^5.7.0" } }
EOF
  cat > "$dir/index.html" << 'EOF'
<!DOCTYPE html><html lang="en"><head><meta charset="UTF-8"/><meta name="viewport" content="width=device-width,initial-scale=1.0"/><title>Vue App</title></head><body><div id="app"></div><script type="module" src="/src/main.ts"></script></body></html>
EOF
  cat > "$dir/src/main.ts" << 'EOF'
import { createApp } from 'vue'
import App from './App.vue'
createApp(App).mount('#app')
EOF
  cat > "$dir/src/App.vue" << 'EOF'
<template><h1>Hello Vue</h1></template>
EOF
  cat > "$dir/src/vite-env.d.ts" << 'EOF'
/// <reference types="vite/client" />
EOF
  generate_vite_config "$dir" "vue from '@vitejs/plugin-vue'"
done

# --- Svelte ---
for ver in v4.0 v4.2 v5.0 v5.1; do
  dir="$ADAPTERS/frontend/svelte/$ver"
  case $ver in
    v4.0) sv="^4.0.0"; plugin="^3.0.0" ;;
    v4.2) sv="^4.2.0"; plugin="^3.0.0" ;;
    v5.0) sv="^5.0.0"; plugin="^5.0.0" ;;
    v5.1) sv="^5.1.0"; plugin="^5.0.0" ;;
  esac
  mkdir -p "$dir/src"
  cat > "$dir/package.json" << EOF
{ "name": "svelte-app", "version": "1.0.0", "private": true, "scripts": { "dev": "vite", "build": "vite build" }, "dependencies": { "svelte": "$sv" }, "devDependencies": { "@sveltejs/vite-plugin-svelte": "$plugin", "vite": "^6.2.0", "typescript": "^5.7.0" } }
EOF
  cat > "$dir/index.html" << 'EOF'
<!DOCTYPE html><html lang="en"><head><meta charset="UTF-8"/><meta name="viewport" content="width=device-width,initial-scale=1.0"/><title>Svelte</title></head><body><div id="app"></div><script type="module" src="/src/main.ts"></script></body></html>
EOF
  cat > "$dir/src/main.ts" << 'EOF'
import App from './App.svelte'
const app = new App({ target: document.getElementById('app')! })
export default app
EOF
  cat > "$dir/src/App.svelte" << 'EOF'
<h1>Hello Svelte</h1>
EOF
  generate_vite_config "$dir" "svelte from '@sveltejs/vite-plugin-svelte'"
done

# --- Astro ---
for ver in v4.0 v4.5 v4.16 v5.0; do
  dir="$ADAPTERS/frontend/astro/$ver"
  case $ver in
    v4.0) av="^4.0.0" ;;
    v4.5) av="^4.5.0" ;;
    v4.16) av="^4.16.0" ;;
    v5.0) av="^5.0.0" ;;
  esac
  mkdir -p "$dir/src/pages" "$dir/public"
  cat > "$dir/package.json" << EOF
{ "name": "astro-app", "version": "1.0.0", "private": true, "scripts": { "dev": "astro dev", "build": "astro build", "preview": "astro preview" }, "dependencies": { "astro": "$av" } }
EOF
  cat > "$dir/astro.config.mjs" << 'EOF'
import { defineConfig } from 'astro/config'
export default defineConfig({})
EOF
  cat > "$dir/src/pages/index.astro" << 'EOF'
---
<html lang="en"><head><meta charset="UTF-8"/><title>Astro</title></head><body><h1>Hello Astro</h1></body></html>
---
EOF
  cat > "$dir/tsconfig.json" << 'EOF'
{ "compilerOptions": { "module": "ESNext", "moduleResolution": "bundler", "strict": true }, "include": ["src"] }
EOF
done

# --- Nuxt ---
for ver in v3.12 v3.13 v3.14 v3.15; do
  dir="$ADAPTERS/frontend/nuxt/$ver"
  case $ver in
    v3.12) nv="^3.12.0" ;;
    v3.13) nv="^3.13.0" ;;
    v3.14) nv="^3.14.0" ;;
    v3.15) nv="^3.15.0" ;;
  esac
  mkdir -p "$dir/app" "$dir/public"
  cat > "$dir/package.json" << EOF
{ "name": "nuxt-app", "version": "1.0.0", "private": true, "scripts": { "dev": "nuxt dev", "build": "nuxt build", "preview": "nuxt preview" }, "dependencies": { "nuxt": "$nv" } }
EOF
  cat > "$dir/nuxt.config.ts" << 'EOF'
export default defineNuxtConfig({})
EOF
  cat > "$dir/tsconfig.json" << 'EOF'
{ "compilerOptions": { "module": "ESNext", "moduleResolution": "bundler", "strict": true } }
EOF
  cat > "$dir/app/app.vue" << 'EOF'
<template><div><h1>Hello Nuxt</h1></div></template>
EOF
done

# --- Solid ---
for ver in v1.8 v1.9 v1.10 v1.11; do
  dir="$ADAPTERS/frontend/solid/$ver"
  case $ver in
    v1.8) sv="^1.8.0" ;;
    v1.9) sv="^1.9.0" ;;
    v1.10) sv="^1.10.0" ;;
    v1.11) sv="^1.11.0" ;;
  esac
  mkdir -p "$dir/src"
  cat > "$dir/package.json" << EOF
{ "name": "solid-app", "version": "1.0.0", "private": true, "scripts": { "dev": "vite", "build": "vite build" }, "dependencies": { "solid-js": "$sv" }, "devDependencies": { "vite": "^6.2.0", "vite-plugin-solid": "^2.11.0", "typescript": "^5.7.0" } }
EOF
  cat > "$dir/index.html" << 'EOF'
<!DOCTYPE html><html lang="en"><head><meta charset="UTF-8"/><meta name="viewport" content="width=device-width,initial-scale=1.0"/><title>Solid</title></head><body><div id="root"></div><script type="module" src="/src/index.tsx"></script></body></html>
EOF
  cat > "$dir/src/index.tsx" << 'EOF'
import { render } from 'solid-js/web'
import App from './App'
render(() => <App />, document.getElementById('root')!)
EOF
  cat > "$dir/src/App.tsx" << 'EOF'
export default function App() { return <h1>Hello Solid</h1> }
EOF
  cat > "$dir/vite.config.ts" << 'EOF'
import { defineConfig } from 'vite'
import solid from 'vite-plugin-solid'
export default defineConfig({ plugins: [solid()] })
EOF
done

# --- SvelteKit ---
for ver in v1.0 v2.0 v2.5 v2.16; do
  dir="$ADAPTERS/frontend/sveltekit/$ver"
  case $ver in
    v1.0) sv="^1.0.0"; kit="^1.0.0"; adapter="^1.0.0" ;;
    v2.0) sv="^4.0.0"; kit="^2.0.0"; adapter="^2.0.0" ;;
    v2.5) sv="^4.2.0"; kit="^2.5.0"; adapter="^3.0.0" ;;
    v2.16) sv="^5.0.0"; kit="^2.16.0"; adapter="^4.0.0" ;;
  esac
  mkdir -p "$dir/src/routes" "$dir/src/lib" "$dir/static"
  cat > "$dir/package.json" << EOF
{ "name": "sveltekit-app", "version": "1.0.0", "private": true, "scripts": { "dev": "vite dev", "build": "vite build", "preview": "vite preview" }, "dependencies": { "@sveltejs/kit": "$kit" }, "devDependencies": { "svelte": "$sv", "@sveltejs/adapter-auto": "$adapter", "vite": "^6.2.0", "typescript": "^5.7.0" } }
EOF
  cat > "$dir/svelte.config.js" << 'EOF'
import adapter from '@sveltejs/adapter-auto'
const config = { kit: { adapter: adapter() } }
export default config
EOF
  cat > "$dir/vite.config.ts" << 'EOF'
import { sveltekit } from '@sveltejs/kit/vite'
export default { plugins: [sveltekit()] }
EOF
  cat > "$dir/src/app.html" << 'EOF'
<!DOCTYPE html><html lang="en"><head><meta charset="UTF-8"/><meta name="viewport" content="width=device-width"/><title>SvelteKit</title></head><body><div id="svelte">%sveltekit.body%</div></body></html>
EOF
  cat > "$dir/src/routes/+page.svelte" << 'EOF'
<h1>Hello SvelteKit</h1>
EOF
done

# --- Preact ---
for ver in v10.19 v10.20 v10.24 v10.26; do
  dir="$ADAPTERS/frontend/preact/$ver"
  case $ver in
    v10.19) pv="^10.19.0" ;;
    v10.20) pv="^10.20.0" ;;
    v10.24) pv="^10.24.0" ;;
    v10.26) pv="^10.26.0" ;;
  esac
  mkdir -p "$dir/src"
  cat > "$dir/package.json" << EOF
{ "name": "preact-app", "version": "1.0.0", "private": true, "scripts": { "dev": "vite", "build": "vite build" }, "dependencies": { "preact": "$pv" }, "devDependencies": { "@preact/preset-vite": "^2.9.0", "vite": "^6.2.0", "typescript": "^5.7.0" } }
EOF
  cat > "$dir/src/main.tsx" << 'EOF'
import { render } from 'preact'
import App from './app'
render(<App />, document.getElementById('app')!)
EOF
  cat > "$dir/src/app.tsx" << 'EOF'
export default function App() { return <h1>Hello Preact</h1> }
EOF
  cat > "$dir/index.html" << 'EOF'
<!DOCTYPE html><html lang="en"><head><meta charset="UTF-8"/><meta name="viewport" content="width=device-width,initial-scale=1.0"/><title>Preact</title></head><body><div id="app"></div><script type="module" src="/src/main.tsx"></script></body></html>
EOF
  cat > "$dir/vite.config.ts" << 'EOF'
import { defineConfig } from 'vite'
import preact from '@preact/preset-vite'
export default defineConfig({ plugins: [preact()] })
EOF
done

# --- Lit ---
for ver in v3.0 v3.1 v3.2 v3.3; do
  dir="$ADAPTERS/frontend/lit/$ver"
  case $ver in
    v3.0) lv="^3.0.0" ;;
    v3.1) lv="^3.1.0" ;;
    v3.2) lv="^3.2.0" ;;
    v3.3) lv="^3.3.0" ;;
  esac
  mkdir -p "$dir/src"
  cat > "$dir/package.json" << EOF
{ "name": "lit-app", "version": "1.0.0", "private": true, "scripts": { "dev": "vite", "build": "vite build" }, "dependencies": { "lit": "$lv" }, "devDependencies": { "vite": "^6.2.0", "typescript": "^5.7.0" } }
EOF
  cat > "$dir/src/my-element.ts" << 'EOF'
import { LitElement, html } from 'lit'
import { customElement } from 'lit/decorators.js'
@customElement('my-element')
export class MyElement extends LitElement { render() { return html`<h1>Hello Lit</h1>` } }
EOF
  cat > "$dir/index.html" << 'EOF'
<!DOCTYPE html><html lang="en"><head><meta charset="UTF-8"/><meta name="viewport" content="width=device-width,initial-scale=1.0"/><title>Lit</title></head><body><my-element></my-element><script type="module" src="/src/my-element.ts"></script></body></html>
EOF
done

# --- Alpine ---
for ver in v3.12 v3.13 v3.14 v3.15; do
  dir="$ADAPTERS/frontend/alpine/$ver"
  case $ver in
    v3.12) av="^3.12.0" ;;
    v3.13) av="^3.13.0" ;;
    v3.14) av="^3.14.0" ;;
    v3.15) av="^3.15.0" ;;
  esac
  cat > "$dir/package.json" << EOF
{ "name": "alpine-app", "version": "1.0.0", "private": true, "scripts": {} }
EOF
  cat > "$dir/index.html" << EOF
<!DOCTYPE html><html lang="en"><head><meta charset="UTF-8"/><meta name="viewport" content="width=device-width,initial-scale=1.0"/><title>Alpine</title><script src="https://cdn.jsdelivr.net/npm/alpinejs@$av/dist/cdn.min.js" defer></script></head><body><div x-data="{ msg: 'Hello Alpine' }"><h1 x-text="msg"></h1></div></body></html>
EOF
done

# --- Qwik ---
for ver in v1.0 v1.5 v1.9 v1.11; do
  dir="$ADAPTERS/frontend/qwik/$ver"
  case $ver in
    v1.0) qv="^1.0.0" ;;
    v1.5) qv="^1.5.0" ;;
    v1.9) qv="^1.9.0" ;;
    v1.11) qv="^1.11.0" ;;
  esac
  mkdir -p "$dir/src/routes" "$dir/public"
  cat > "$dir/package.json" << EOF
{ "name": "qwik-app", "version": "1.0.0", "private": true, "scripts": { "dev": "vite", "build": "vite build" }, "dependencies": { "@builder.io/qwik": "$qv", "@builder.io/qwik-city": "$qv" }, "devDependencies": { "vite": "^6.2.0", "typescript": "^5.7.0" } }
EOF
  cat > "$dir/src/root.tsx" << 'EOF'
export default function Root() { return <html><head><title>Qwik</title></head><body><h1>Hello Qwik</h1></body></html> }
EOF
  cat > "$dir/src/routes/index.tsx" << 'EOF'
export default function Home() { return <h1>Home</h1> }
EOF
done

# --- Angular ---
for ver in v16 v17 v18 v19; do
  dir="$ADAPTERS/frontend/angular/$ver"
  case $ver in
    v16) av="^16.0.0" ;;
    v17) av="^17.0.0" ;;
    v18) av="^18.0.0" ;;
    v19) av="^19.0.0" ;;
  esac
  mkdir -p "$dir/src/app"
  cat > "$dir/package.json" << EOF
{ "name": "angular-app", "version": "1.0.0", "private": true, "scripts": { "ng": "ng", "start": "ng serve", "build": "ng build" }, "dependencies": { "@angular/core": "$av", "@angular/platform-browser": "$av", "@angular/router": "$av", "zone.js": "^0.14.0", "rxjs": "^7.8.0" }, "devDependencies": { "@angular/cli": "$av", "@angular/compiler-cli": "$av", "typescript": "^5.4.0" } }
EOF
  cat > "$dir/src/main.ts" << 'EOF'
import { platformBrowserDynamic } from '@angular/platform-browser-dynamic'
import { AppModule } from './app/app.module'
platformBrowserDynamic().bootstrapModule(AppModule)
EOF
  cat > "$dir/src/app/app.module.ts" << 'EOF'
import { NgModule } from '@angular/core'
import { BrowserModule } from '@angular/platform-browser'
import { AppComponent } from './app.component'
@NgModule({ declarations: [AppComponent], imports: [BrowserModule], bootstrap: [AppComponent] })
export class AppModule {}
EOF
  cat > "$dir/src/app/app.component.ts" << 'EOF'
import { Component } from '@angular/core'
@Component({ selector: 'app-root', template: '<h1>Hello Angular</h1>' })
export class AppComponent {}
EOF
  cat > "$dir/src/index.html" << 'EOF'
<!DOCTYPE html><html><head><base href="/"/></head><body><app-root></app-root></body></html>
EOF
done

# --- Remix ---
for ver in v2.0 v2.10 v2.15 v2.16; do
  dir="$ADAPTERS/frontend/remix/$ver"
  case $ver in
    v2.0) rv="^2.0.0"; rr="^6.20.0" ;;
    v2.10) rv="^2.10.0"; rr="^6.22.0" ;;
    v2.15) rv="^2.15.0"; rr="^7.0.0" ;;
    v2.16) rv="^2.16.0"; rr="^7.0.0" ;;
  esac
  mkdir -p "$dir/app/routes" "$dir/public"
  cat > "$dir/package.json" << EOF
{ "name": "remix-app", "version": "1.0.0", "private": true, "scripts": { "dev": "remix dev", "build": "remix build" }, "dependencies": { "@remix-run/react": "$rv", "@remix-run/node": "$rv", "@remix-run/serve": "$rv", "react": "$rr", "react-dom": "$rr" }, "devDependencies": { "typescript": "^5.7.0", "vite": "^6.2.0" } }
EOF
  cat > "$dir/app/root.tsx" << 'EOF'
import { Outlet } from '@remix-run/react'
export default function Root() { return <html><body><Outlet/></body></html> }
EOF
  cat > "$dir/app/routes/_index.tsx" << 'EOF'
export default function Index() { return <h1>Hello Remix</h1> }
EOF
done

# --- SolidStart ---
for ver in v0.6 v1.0 v1.1 v1.2; do
  dir="$ADAPTERS/frontend/solidstart/$ver"
  case $ver in
    v0.6) sv="^1.8.0"; ss="^0.6.0"; vinxi="^0.2.0" ;;
    v1.0) sv="^1.9.0"; ss="^1.0.0"; vinxi="^0.4.0" ;;
    v1.1) sv="^1.9.0"; ss="^1.1.0"; vinxi="^0.5.0" ;;
    v1.2) sv="^1.9.0"; ss="^1.2.0"; vinxi="^0.5.0" ;;
  esac
  mkdir -p "$dir/src/routes" "$dir/public"
  cat > "$dir/package.json" << EOF
{ "name": "solidstart-app", "version": "1.0.0", "private": true, "scripts": { "dev": "vinxi dev", "build": "vinxi build" }, "dependencies": { "solid-js": "$sv", "@solidjs/start": "$ss", "vinxi": "$vinxi" }, "devDependencies": { "vite": "^6.2.0", "typescript": "^5.7.0" } }
EOF
  cat > "$dir/app.config.ts" << 'EOF'
import { defineConfig } from '@solidjs/start/config'
export default defineConfig({})
EOF
  cat > "$dir/src/root.tsx" << 'EOF'
import { Routes, FileRoutes } from '@solidjs/start'
export default function Root() { return <html><body><Routes><FileRoutes/></Routes></body></html> }
EOF
  cat > "$dir/src/routes/index.tsx" << 'EOF'
export default function Home() { return <h1>Hello SolidStart</h1> }
EOF
done

# --- TanStack Start ---
for ver in v1.0 v1.37 v1.56 v1.66; do
  dir="$ADAPTERS/frontend/tanstack_start/$ver"
  case $ver in
    v1.0) tr="^1.0.0"; ts="^1.0.0" ;;
    v1.37) tr="^1.37.0"; ts="^1.37.0" ;;
    v1.56) tr="^1.56.0"; ts="^1.56.0" ;;
    v1.66) tr="^1.66.0"; ts="^1.66.0" ;;
  esac
  mkdir -p "$dir/app/routes"
  cat > "$dir/package.json" << EOF
{ "name": "tanstack-app", "version": "1.0.0", "private": true, "scripts": { "dev": "vinxi dev" }, "dependencies": { "@tanstack/react-router": "$tr", "@tanstack/start": "$ts", "react": "^19.0.0", "react-dom": "^19.0.0", "vinxi": "^0.5.0" }, "devDependencies": { "typescript": "^5.7.0" } }
EOF
  cat > "$dir/app/routes/index.tsx" << 'EOF'
export default function Home() { return <h1>Hello TanStack</h1> }
EOF
done

# --- Waku ---
for ver in v0.19 v0.20 v0.21 v0.22; do
  dir="$ADAPTERS/frontend/waku/$ver"
  case $ver in
    v0.19) wv="^0.19.0" ;;
    v0.20) wv="^0.20.0" ;;
    v0.21) wv="^0.21.0" ;;
    v0.22) wv="^0.22.0" ;;
  esac
  mkdir -p "$dir/src"
  cat > "$dir/package.json" << EOF
{ "name": "waku-app", "version": "1.0.0", "private": true, "scripts": { "dev": "waku dev" }, "dependencies": { "waku": "$wv", "react": "^19.0.0", "react-dom": "^19.0.0" }, "devDependencies": { "typescript": "^5.7.0" } }
EOF
  cat > "$dir/src/main.tsx" << 'EOF'
import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import App from './App'
createRoot(document.getElementById('root')!).render(<StrictMode><App/></StrictMode>)
EOF
  cat > "$dir/src/App.tsx" << 'EOF'
export default function App() { return <h1>Hello Waku</h1> }
EOF
done

# --- Marko ---
for ver in v5.33 v5.34 v6.0 v6.1; do
  dir="$ADAPTERS/frontend/marko/$ver"
  case $ver in
    v5.33) mv="^5.33.0" ;;
    v5.34) mv="^5.34.0" ;;
    v6.0) mv="^6.0.0" ;;
    v6.1) mv="^6.1.0" ;;
  esac
  cat > "$dir/package.json" << EOF
{ "name": "marko-app", "version": "1.0.0", "private": true, "scripts": { "dev": "marko" }, "dependencies": { "marko": "$mv" } }
EOF
  cat > "$dir/index.html" << 'EOF'
<!DOCTYPE html><html><body><h1>Hello Marko</h1></body></html>
EOF
done

# --- Fresh --- (Deno-based, minimal)
for ver in v1.0 v1.5 v1.6 v1.7; do
  dir="$ADAPTERS/frontend/fresh/$ver"
  mkdir -p "$dir/routes"
  cat > "$dir/deno.json" << 'EOF'
{ "tasks": { "start": "deno run -A main.ts" } }
EOF
  cat > "$dir/main.ts" << 'EOF'
import { start } from '$fresh/server.ts'
import routes from './routes/index.tsx'
start({ routes })
EOF
  cat > "$dir/routes/index.tsx" << 'EOF'
export default function Home() { return <h1>Hello Fresh</h1> }
EOF
done

# --- Stencil ---
for ver in v4.0 v4.7 v4.20 v4.22; do
  dir="$ADAPTERS/frontend/stencil/$ver"
  case $ver in
    v4.0) sv="^4.0.0" ;;
    v4.7) sv="^4.7.0" ;;
    v4.20) sv="^4.20.0" ;;
    v4.22) sv="^4.22.0" ;;
  esac
  mkdir -p "$dir/src/components/my-component"
  cat > "$dir/package.json" << EOF
{ "name": "stencil-app", "version": "1.0.0", "private": true, "scripts": { "dev": "stencil dev" }, "dependencies": { "@stencil/core": "$sv" } }
EOF
done

# --- Redwood ---
for ver in v6.0 v7.0 v7.6 v8.0; do
  dir="$ADAPTERS/frontend/redwood/$ver"
  case $ver in
    v6.0) rw="^6.0.0" ;;
    v7.0) rw="^7.0.0" ;;
    v7.6) rw="^7.6.0" ;;
    v8.0) rw="^8.0.0" ;;
  esac
  cat > "$dir/package.json" << EOF
{ "name": "redwood-app", "version": "1.0.0", "private": true, "scripts": { "dev": "rw dev" }, "dependencies": { "@redwoodjs/core": "$rw" } }
EOF
done

# --- Million ---
for ver in v3.0 v3.1 v3.2 v3.3; do
  dir="$ADAPTERS/frontend/million/$ver"
  case $ver in
    v3.0) mv="^3.0.0" ;;
    v3.1) mv="^3.1.0" ;;
    v3.2) mv="^3.2.0" ;;
    v3.3) mv="^3.3.0" ;;
  esac
  cat > "$dir/package.json" << EOF
{ "name": "million-app", "version": "1.0.0", "private": true, "scripts": { "dev": "vite" }, "dependencies": { "million": "$mv", "react": "^19.0.0", "react-dom": "^19.0.0" }, "devDependencies": { "vite": "^6.2.0" } }
EOF
done

# --- Mithril, Aurelia, Ember, Riot, Hyperapp, Stimulus ---
for fw in mithril aurelia ember riot hyperapp stimulus; do
  for ver in $(ls "$ADAPTERS/frontend/$fw/"); do
    dir="$ADAPTERS/frontend/$fw/$ver"
    cat > "$dir/package.json" << EOF
{ "name": "$fw-app", "version": "1.0.0", "private": true, "scripts": {} }
EOF
    cat > "$dir/index.html" << EOF
<!DOCTYPE html><html><body><h1>Hello $(echo $fw | sed 's/.*/\u&/')</h1></body></html>
EOF
  done
done

echo "FRONTEND DONE"
