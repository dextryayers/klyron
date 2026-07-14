use std::collections::HashMap;
use std::path::Path;
use async_trait::async_trait;
use anyhow::Result;
use crate::{FrameworkAdapter, BuildOptions, ScaffoldOptions, FrameworkKind};

pub struct RailsAdapter;

#[async_trait]
impl FrameworkAdapter for RailsAdapter {
    fn name(&self) -> &'static str { "rails" }

    fn detect(&self, dir: &Path) -> bool {
        dir.join("bin/rails").exists()
    }

    fn supported_versions(&self) -> Vec<&'static str> { vec!["7.1", "8.0"] }
    fn default_version(&self) -> &'static str { "8.0" }
    fn kind(&self) -> FrameworkKind { FrameworkKind::Polyglot }

    async fn dev(&self, dir: &Path, port: Option<u16>) -> Result<()> {
        let mut cmd = tokio::process::Command::new("bin/rails");
        cmd.args(["server"]).current_dir(dir);
        if let Some(p) = port { cmd.env("PORT", p.to_string()); }
        cmd.status().await?; Ok(())
    }

    async fn build(&self, dir: &Path, _opts: BuildOptions) -> Result<()> {
        tokio::process::Command::new("bin/rails").args(["assets:precompile"]).current_dir(dir).status().await?;
        Ok(())
    }

    async fn test(&self, dir: &Path, _filter: Option<&str>) -> Result<()> {
        tokio::process::Command::new("bundle").args(["exec", "rspec"]).current_dir(dir).status().await?;
        Ok(())
    }

    async fn lint(&self, dir: &Path, _fix: bool) -> Result<()> {
        tokio::process::Command::new("bundle").args(["exec", "rubocop"]).current_dir(dir).status().await?;
        Ok(())
    }

    async fn format(&self, dir: &Path, write: bool) -> Result<()> {
        if write {
            tokio::process::Command::new("bundle").args(["exec", "rubocop", "-A"]).current_dir(dir).status().await?;
        } else {
            tokio::process::Command::new("bundle").args(["exec", "rubocop"]).current_dir(dir).status().await?;
        }
        Ok(())
    }

    async fn scaffold(&self, name: &str, options: ScaffoldOptions) -> Result<()> {
        let project_dir = options.dir.join(name);
        std::fs::create_dir_all(&project_dir)?;
        std::fs::create_dir_all(project_dir.join("app/controllers"))?;
        std::fs::create_dir_all(project_dir.join("app/models"))?;
        std::fs::create_dir_all(project_dir.join("app/views/layouts"))?;
        std::fs::create_dir_all(project_dir.join("app/views/home"))?;
        std::fs::create_dir_all(project_dir.join("app/helpers"))?;
        std::fs::create_dir_all(project_dir.join("app/javascript"))?;
        std::fs::create_dir_all(project_dir.join("app/assets/stylesheets"))?;
        std::fs::create_dir_all(project_dir.join("config/environments"))?;
        std::fs::create_dir_all(project_dir.join("db/migrate"))?;
        std::fs::create_dir_all(project_dir.join("bin"))?;
        std::fs::create_dir_all(project_dir.join("spec/models"))?;
        std::fs::create_dir_all(project_dir.join("spec/requests"))?;

        let vars = &options.template_vars;

        std::fs::write(project_dir.join("Gemfile"),
            klyron_template::TemplateEngine::render(r#"source 'https://rubygems.org'

ruby '>= 3.2.0'

gem 'rails', '~> 8.0'
gem 'pg', '~> 1.5'
gem 'puma', '~> 6.0'
gem 'importmap-rails', '~> 2.0'
gem 'turbo-rails', '~> 2.0'
gem 'stimulus-rails', '~> 1.3'
gem 'jbuilder', '~> 2.11'
gem 'bootsnap', require: false

group :development, :test do
  gem 'rspec-rails', '~> 7.0'
  gem 'rubocop', '~> 1.60'
  gem 'rubocop-rails', '~> 2.25'
  gem 'debug', platforms: [:mri, :mingw, :x64_mingw]
end

group :development do
  gem 'web-console', '~> 4.2'
end

group :test do
  gem 'capybara', '~> 3.40'
  gem 'selenium-webdriver', '~> 4.10'
end
"#, vars))?;

        std::fs::write(project_dir.join("Rakefile"),
            r#"require_relative 'config/application'
Rails.application.load_tasks
"#)?;

        std::fs::write(project_dir.join("config.ru"),
            r#"require_relative 'config/environment'
run Rails.application
"#)?;

        std::fs::write(project_dir.join("package.json"),
            r#"{
  "name": "app",
  "private": true,
  "dependencies": { "@hotwired/stimulus": "^3.2", "@hotwired/turbo-rails": "^8.0" },
  "scripts": { "build": "esbuild app/javascript/*.* --bundle --sourcemap --outdir=app/assets/builds" }
}"#)?;

        std::fs::write(project_dir.join("config/application.rb"),
            klyron_template::TemplateEngine::render(r#"require_relative 'boot'
require 'rails/all'
Bundler.require(*Rails.groups)

module {{ name | capitalize }}
  class Application < Rails::Application
    config.load_defaults 8.0
    config.autoload_lib(ignore: %w[assets tasks])
  end
end
"#, vars))?;

        std::fs::write(project_dir.join("config/database.yml"),
            r#"default: &default
  adapter: postgresql
  encoding: unicode
  pool: 5

development:
  <<: *default
  database: app_development

test:
  <<: *default
  database: app_test

production:
  <<: *default
  database: app_production
  username: app
  password: <%= ENV['APP_DATABASE_PASSWORD'] %>
"#)?;

        std::fs::write(project_dir.join("config/routes.rb"),
            r#"Rails.application.routes.draw do
  root 'home#index'
end
"#)?;

        std::fs::write(project_dir.join("config/environments/development.rb"),
            r#"require 'active_support/core_ext/integer/time'
Rails.application.configure do
  config.enable_reloading = true
  config.eager_load = false
  config.consider_all_requests_local = true
  config.server_timing = true
end
"#)?;

        std::fs::write(project_dir.join("config/environments/production.rb"),
            r#"require 'active_support/core_ext/integer/time'
Rails.application.configure do
  config.enable_reloading = false
  config.eager_load = true
  config.consider_all_requests_local = false
end
"#)?;

        std::fs::write(project_dir.join("config/environments/test.rb"),
            r#"require 'active_support/core_ext/integer/time'
Rails.application.configure do
  config.enable_reloading = false
  config.eager_load = false
  config.consider_all_requests_local = true
  config.cache_classes = true
end
"#)?;

        std::fs::write(project_dir.join("app/controllers/application_controller.rb"),
            r#"class ApplicationController < ActionController::Base
end
"#)?;

        std::fs::write(project_dir.join("app/controllers/home_controller.rb"),
            r#"class HomeController < ApplicationController
  def index
  end
end
"#)?;

        std::fs::write(project_dir.join("app/models/application_record.rb"),
            r#"class ApplicationRecord < ActiveRecord::Base
  primary_abstract_class
end
"#)?;

        std::fs::write(project_dir.join("app/models/user.rb"),
            r#"class User < ApplicationRecord
  has_secure_password
  validates :email, presence: true, uniqueness: true
end
"#)?;

        std::fs::write(project_dir.join("app/views/layouts/application.html.erb"),
            klyron_template::TemplateEngine::render(r#"<!DOCTYPE html>
<html><head><title>{{ name }}</title><meta name="viewport" content="width=device-width,initial-scale=1"><%= csrf_meta_tags %><%= csp_meta_tag %><%= stylesheet_link_tag "application", "data-turbo-track": "reload" %></head>
<body><%= yield %></body></html>
"#, vars))?;

        std::fs::write(project_dir.join("app/views/home/index.html.erb"),
            klyron_template::TemplateEngine::render(r#"<h1>Welcome to {{ name }}</h1>
"#, vars))?;

        std::fs::write(project_dir.join("app/helpers/application_helper.rb"),
            r#"module ApplicationHelper
end
"#)?;

        std::fs::write(project_dir.join("app/javascript/application.js"),
            r#"import '@hotwired/turbo-rails'
import '@hotwired/stimulus'
"#)?;

        std::fs::write(project_dir.join("app/assets/stylesheets/application.css"),
            r#"* { margin: 0; padding: 0; box-sizing: border-box; }
body { font-family: system-ui, sans-serif; min-height: 100vh; }
"#)?;

        std::fs::write(project_dir.join("db/migrate/20240101000001_create_users.rb"),
            r#"class CreateUsers < ActiveRecord::Migration[8.0]
  def change
    create_table :users do |t|
      t.string :email, null: false
      t.string :password_digest, null: false
      t.timestamps
    end
    add_index :users, :email, unique: true
  end
end
"#)?;

        std::fs::write(project_dir.join("db/seeds.rb"),
            r#"puts 'Seeding database...'
"#)?;

        std::fs::write(project_dir.join("bin/rails"),
            r#"#!/usr/bin/env ruby
APP_PATH = File.expand_path('../config/application', __dir__)
require_relative '../config/boot'
require 'rails/commands'
"#)?;

        std::fs::write(project_dir.join("bin/setup"),
            r#"#!/usr/bin/env ruby
require 'fileutils'
FileUtils.chdir APP_ROOT do
  system! 'gem install bundler --conservative'
  system('bundle check') || system!('bundle install')
  system! 'bin/rails db:prepare'
end
"#)?;

        std::fs::write(project_dir.join("spec/spec_helper.rb"),
            r#"RSpec.configure do |config|
  config.expect_with :rspec do |expectations|
    expectations.include_chain_clauses_in_custom_matcher_descriptions = true
  end
  config.mock_with :rspec
end
"#)?;

        std::fs::write(project_dir.join("spec/rails_helper.rb"),
            r#"ENV['RAILS_ENV'] ||= 'test'
require_relative '../config/environment'
abort('The Rails environment is running in production mode!') if Rails.env.production?
require 'rspec/rails'
begin
  ActiveRecord::Migration.maintain_test_schema!
rescue ActiveRecord::PendingMigrationError => e
  abort e.to_s.strip
end
RSpec.configure do |config|
  config.fixture_paths = [Rails.root.join('spec/fixtures')]
  config.use_transactional_fixtures = true
  config.infer_spec_type_from_file_location!
end
"#)?;

        std::fs::write(project_dir.join("spec/models/user_spec.rb"),
            r#"require 'rails_helper'
RSpec.describe User, type: :model do
  it 'is valid with valid attributes' do
    user = User.new(email: 'test@example.com', password: 'password')
    expect(user).to be_valid
  end
end
"#)?;

        std::fs::write(project_dir.join("spec/requests/home_spec.rb"),
            r#"require 'rails_helper'
RSpec.describe 'Home', type: :request do
  describe 'GET /' do
    it 'returns http success' do
      get '/'
      expect(response).to have_http_status(:success)
    end
  end
end
"#)?;

        std::fs::write(project_dir.join(".gitignore"),
            "node_modules\ntmp\nlog\n.DS_Store\n*.sqlite3\n")?;

        std::fs::write(project_dir.join("README.md"),
            klyron_template::TemplateEngine::render(r#"# {{ name }}

Rails application

## Getting Started

bundle install
bin/rails db:create db:migrate
bin/rails server
"#, vars))?;

        Ok(())
    }
}
