import { readFileSync, statSync, writeFileSync, mkdirSync } from 'fs'
import { join } from 'path'

export function createAdapter() {
  return {
    name: 'rails',
    detect(dir) {
      try {
        const gemfile = readFileSync(join(dir, 'Gemfile'), 'utf-8')
        if (/rails\s*['"~>]/.test(gemfile)) return true
      } catch { /* ignore */ }
      try {
        return statSync(join(dir, 'config', 'application.rb')).isFile()
      } catch { return false }
    },
    supportedVersions: ['7.1', '7.2', '8.0'],
    defaultVersion: '8.0',
    kind: 'Fullstack',

    async dev(dir, port) {
      const { spawn } = await import('child_process')
      const proc = spawn('rails', ['server', '-b', '0.0.0.0', '-p', String(port || 3000)], { cwd: dir, stdio: 'inherit' })
      return new Promise((resolve, reject) => {
        proc.on('close', (code) => code === 0 ? resolve() : reject(new Error(`Dev server exited with code ${code}`)))
      })
    },

    async build(dir) {
      const { execFile } = await import('child_process')
      await execFile('rails', ['assets:precompile'], { cwd: dir })
    },

    async test(dir) {
      const { execFile } = await import('child_process')
      try {
        await execFile('rails', ['test'], { cwd: dir })
      } catch {
        await execFile('bundle', ['exec', 'rspec'], { cwd: dir })
      }
    },

    async lint(dir) {
      const { execFile } = await import('child_process')
      try {
        await execFile('bundle', ['exec', 'rubocop'], { cwd: dir })
      } catch {
        // no linter configured
      }
    },

    async format(dir, write) {
      const { execFile } = await import('child_process')
      try {
        await execFile('bundle', ['exec', 'rubocop', write ? '-A' : '--check'], { cwd: dir })
      } catch {
        // no formatter configured
      }
    },

    scaffold(name, options) {
      return scaffoldRails(name, options)
    }
  }
}

function readPackageJson(dir) {
  try {
    return JSON.parse(readFileSync(join(dir, 'package.json'), 'utf-8'))
  } catch { return null }
}

function existsSync(p) {
  try { return statSync(p).isFile() } catch { return false }
}

async function scaffoldRails(name, options) {
  const projectDir = join(options.dir || process.cwd(), name)
  mkdirSync(join(projectDir, 'app', 'controllers'), { recursive: true })
  mkdirSync(join(projectDir, 'app', 'models'), { recursive: true })
  mkdirSync(join(projectDir, 'app', 'views'), { recursive: true })
  mkdirSync(join(projectDir, 'config'), { recursive: true })
  mkdirSync(join(projectDir, 'db', 'migrate'), { recursive: true })
  mkdirSync(join(projectDir, 'public'), { recursive: true })
  mkdirSync(join(projectDir, 'log'), { recursive: true })
  mkdirSync(join(projectDir, 'tmp'), { recursive: true })

  writeFileSync(join(projectDir, 'Gemfile'), `source 'https://rubygems.org'

ruby '>= 3.2'

gem 'rails', '~> 8.0'
gem 'pg', '~> 1.5'
gem 'puma', '~> 6.4'

group :development, :test do
  gem 'rspec-rails', '~> 7.0'
  gem 'rubocop', '~> 1.60'
end
`)

  writeFileSync(join(projectDir, 'config', 'routes.rb'), `Rails.application.routes.draw do
  root to: 'home#index'

  get '/health', to: 'health#show'
end
`)

  writeFileSync(join(projectDir, 'config', 'database.yml'), `default: &default
  adapter: postgresql
  encoding: unicode
  pool: <%= ENV.fetch("RAILS_MAX_THREADS") { 5 } %>

development:
  <<: *default
  database: ${name}_development

test:
  <<: *default
  database: ${name}_test

production:
  <<: *default
  database: ${name}_production
  username: ${name}
  password: <%= ENV['${name}_DATABASE_PASSWORD'] %>
`)

  writeFileSync(join(projectDir, 'config', 'application.rb'), `require_relative 'boot'
require 'rails/all'

Bundler.require(*Rails.groups)

module ${toPascalCase(name)}
  class Application < Rails::Application
    config.load_defaults 8.0
    config.autoload_lib(ignore: %w(assets tasks))
  end
end
`)

  writeFileSync(join(projectDir, 'config', 'boot.rb'), `ENV['BUNDLE_GEMFILE'] ||= File.expand_path('../Gemfile', __dir__)
require 'bundler/setup'
`)

  writeFileSync(join(projectDir, 'config', 'puma.rb'), `max_threads_count = ENV.fetch("RAILS_MAX_THREADS") { 5 }
min_threads_count = ENV.fetch("RAILS_MIN_THREADS") { max_threads_count }
threads min_threads_count, max_threads_count

port ENV.fetch("PORT") { 3000 }
environment ENV.fetch("RAILS_ENV") { "development" }
pidfile ENV.fetch("PIDFILE") { "tmp/pids/server.pid" }
`)

  writeFileSync(join(projectDir, 'app', 'controllers', 'application_controller.rb'), `class ApplicationController < ActionController::Base
end
`)

  writeFileSync(join(projectDir, 'app', 'controllers', 'home_controller.rb'), `class HomeController < ApplicationController
  def index
    render json: { message: 'Hello from Rails' }
  end
end
`)

  writeFileSync(join(projectDir, 'app', 'controllers', 'health_controller.rb'), `class HealthController < ApplicationController
  def show
    render json: { status: 'ok', timestamp: Time.current.iso8601 }
  end
end
`)

  writeFileSync(join(projectDir, 'app', 'models', 'application_record.rb'), `class ApplicationRecord < ActiveRecord::Base
  primary_abstract_class
end
`)

  writeFileSync(join(projectDir, 'Rakefile'), `require_relative 'config/application'
Rails.application.load_tasks
`)

  writeFileSync(join(projectDir, 'bin', 'rails'), `#!/usr/bin/env ruby
APP_PATH = File.expand_path('../config/application', __dir__)
require_relative '../config/boot'
require 'rails/commands'
`)

  writeFileSync(join(projectDir, 'package.json'), JSON.stringify({
    name, version: '1.0.0', private: true,
    scripts: {
      dev: 'rails server',
      build: 'rails assets:precompile',
      test: 'rails test'
    }
  }, null, 2))

  writeFileSync(join(projectDir, '.gitignore'), 'vendor/bundle\nlog/*.log\ntmp/*\n.DS_Store\npublic/assets\n')
  writeFileSync(join(projectDir, '.prettierrc'), JSON.stringify({ semi: true, singleQuote: true, tabWidth: 2, trailingComma: 'es5', printWidth: 100 }))
  writeFileSync(join(projectDir, 'README.md'), `# ${name}

Ruby on Rails API

## Getting Started

bundle install
rails server
`)
}

function toPascalCase(str) {
  return str.replace(/(?:^|[-_])(\w)/g, (_, c) => c.toUpperCase())
}
