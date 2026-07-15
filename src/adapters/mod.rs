use std::collections::HashMap;
use std::path::Path;

pub mod middleware;
pub mod templates;

pub mod next;
pub mod express;
pub mod react;
pub mod vue;
pub mod svelte;
pub mod astro;
pub mod nuxt;
pub mod hono;
pub mod elysia;
pub mod fastify;
pub mod koa;
pub mod nest;

pub mod orm;

pub trait AdapterTrait: Send + Sync {
    fn name(&self) -> &'static str;
    fn detect(&self, project_dir: &Path) -> bool;
    fn get_config(&self, project_dir: &Path) -> Result<AdapterConfig, AdapterError>;
    fn get_build_command(&self) -> Vec<String>;
    fn get_dev_command(&self) -> Vec<String>;
    fn get_output_dir(&self) -> &'static str;
    fn validate_project(&self, project_dir: &Path) -> Result<Vec<String>, AdapterError>;
    fn get_dependencies(&self) -> Vec<AdapterDep>;
    fn get_template_files(&self) -> Vec<TemplateFile>;
    fn get_middleware_pattern(&self) -> Vec<String>;
    fn get_route_pattern(&self) -> Vec<String>;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AdapterConfig {
    pub name: String,
    pub version: Option<String>,
    pub build_command: Option<String>,
    pub dev_command: Option<String>,
    pub output_dir: Option<String>,
    pub src_dir: Option<String>,
    pub port: Option<u16>,
    pub node_version: Option<String>,
    pub custom: HashMap<String, String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AdapterDep {
    pub name: String,
    pub version: String,
    pub is_dev: bool,
    pub is_optional: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TemplateFile {
    pub source: String,
    pub dest: String,
    pub is_template: bool,
}

#[derive(Debug)]
pub enum AdapterError {
    NotFound(String),
    InvalidConfig(String),
    MissingFile(String),
    Version(String),
    Io(std::io::Error),
}

impl std::fmt::Display for AdapterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdapterError::NotFound(s) => write!(f, "Adapter not found: {s}"),
            AdapterError::InvalidConfig(s) => write!(f, "Invalid config: {s}"),
            AdapterError::MissingFile(s) => write!(f, "Missing file: {s}"),
            AdapterError::Version(s) => write!(f, "Version error: {s}"),
            AdapterError::Io(e) => write!(f, "IO error: {e}"),
        }
    }
}

impl std::error::Error for AdapterError {}

impl From<std::io::Error> for AdapterError {
    fn from(e: std::io::Error) -> Self {
        AdapterError::Io(e)
    }
}

pub struct AdapterRegistry {
    adapters: HashMap<String, Box<dyn AdapterTrait>>,
}

impl AdapterRegistry {
    pub fn new() -> Self {
        Self {
            adapters: HashMap::new(),
        }
    }

    pub fn register(&mut self, adapter: Box<dyn AdapterTrait>) {
        self.adapters.insert(adapter.name().to_string(), adapter);
    }

    pub fn get(&self, name: &str) -> Option<&Box<dyn AdapterTrait>> {
        self.adapters.get(name)
    }

    pub fn list(&self) -> Vec<&str> {
        self.adapters.keys().map(|s| s.as_str()).collect()
    }

    pub fn detect(&self, project_dir: &Path) -> Option<&Box<dyn AdapterTrait>> {
        for adapter in self.adapters.values() {
            if adapter.detect(project_dir) {
                return Some(adapter);
            }
        }
        None
    }

    pub fn register_all(&mut self) {
        self.register(Box::new(next::NextAdapter));
        self.register(Box::new(express::ExpressAdapter));
        self.register(Box::new(react::ReactAdapter));
        self.register(Box::new(vue::VueAdapter));
        self.register(Box::new(svelte::SvelteAdapter));
        self.register(Box::new(astro::AstroAdapter));
        self.register(Box::new(nuxt::NuxtAdapter));
        self.register(Box::new(hono::HonoAdapter));
        self.register(Box::new(elysia::ElysiaAdapter));
        self.register(Box::new(fastify::FastifyAdapter));
        self.register(Box::new(koa::KoaAdapter));
        self.register(Box::new(nest::NestAdapter));
    }
}

impl Default for AdapterRegistry {
    fn default() -> Self {
        Self::new()
    }
}
