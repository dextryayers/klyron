#!/usr/bin/env python3
"""Generate all binding files for 10 klyron crates."""
import os
import shutil

CRATES = [
    "klyron_napi", "klyron_pm", "klyron_registry",
    "klyron_bundler", "klyron_transpiler", "klyron_watcher",
    "klyron_test", "klyron_linter", "klyron_formatter", "klyron_bench"
]

BASE = "/home/aniipid/koding/klyronjs/crates"

def ensure_dir(path):
    os.makedirs(path, exist_ok=True)

def write_file(path, content):
    ensure_dir(os.path.dirname(path))
    with open(path, 'w') as f:
        f.write(content.lstrip('\n'))

# ============================================================
# RUST BINDINGS
# ============================================================

def rust_types_napi():
    return """use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NapiModule {
    pub name: String,
    pub exports: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NapiLoaderConfig {
    pub module_paths: Vec<String>,
    pub cache_enabled: bool,
}

impl Default for NapiLoaderConfig {
    fn default() -> Self {
        Self {
            module_paths: vec!["node_modules".to_string()],
            cache_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NapiVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}
"""

def rust_errors_napi():
    return """use thiserror::Error;

#[derive(Error, Debug)]
pub enum NapiError {
    #[error("module '{0}' not found")]
    ModuleNotFound(String),
    #[error("failed to load native module: {0}")]
    LoadFailed(String),
    #[error("symbol '{0}' not exported")]
    SymbolNotFound(String),
    #[error("version mismatch: expected {expected}, got {got}")]
    VersionMismatch { expected: u32, got: u32 },
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
"""

def rust_builder_napi():
    return """use crate::types::{NapiLoaderConfig, NapiModule};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct NapiLoaderBuilder {
    config: NapiLoaderConfig,
}

impl NapiLoaderBuilder {
    pub fn new() -> Self {
        Self { config: NapiLoaderConfig::default() }
    }

    pub fn module_path(mut self, path: &str) -> Self {
        self.config.module_paths.push(path.to_string());
        self
    }

    pub fn cache_enabled(mut self, enabled: bool) -> Self {
        self.config.cache_enabled = enabled;
        self
    }

    pub fn build(self) -> super::NapiLoader {
        super::NapiLoader::with_config(self.config)
    }
}

impl Default for NapiLoaderBuilder {
    fn default() -> Self { Self::new() }
}
"""

def rust_config_napi():
    return """use crate::types::NapiLoaderConfig;

#[derive(Debug, Clone)]
pub struct NapiConfig {
    pub loader: NapiLoaderConfig,
    pub napi_version: u32,
}

impl Default for NapiConfig {
    fn default() -> Self {
        Self {
            loader: NapiLoaderConfig::default(),
            napi_version: 9,
        }
    }
}

impl NapiConfig {
    pub fn new() -> Self { Self::default() }
    pub fn with_loader(mut self, loader: NapiLoaderConfig) -> Self {
        self.loader = loader;
        self
    }
}
"""

def rust_ffi_napi():
    return """use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[repr(C)]
pub struct napi_module_t {
    pub name: *const c_char,
    pub exports: *const c_char,
    pub napi_version: u32,
}

#[no_mangle]
pub extern "C" fn napi_load_module(name: *const c_char) -> *mut napi_module_t {
    let name_str = unsafe { CStr::from_ptr(name) }.to_string_lossy().into_owned();
    let mut loader = super::NapiLoader::new();
    match loader.load(&name_str) {
        Ok(module) => {
            let c_name = CString::new(module.name.clone()).unwrap();
            let c_exports = CString::new("{}").unwrap();
            Box::into_raw(Box::new(napi_module_t {
                name: c_name.into_raw(),
                exports: c_exports.into_raw(),
                napi_version: 9,
            }))
        }
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn napi_free_module(module: *mut napi_module_t) {
    if !module.is_null() {
        unsafe {
            let _ = CString::from_raw((*module).name as *mut c_char);
            let _ = CString::from_raw((*module).exports as *mut c_char);
            drop(Box::from_raw(module));
        }
    }
}
"""

def rust_wasm_napi():
    return """use wasm_bindgen::prelude::*;
use crate::NapiLoader;

#[wasm_bindgen]
pub struct WasmNapiLoader {
    inner: NapiLoader,
}

#[wasm_bindgen]
impl WasmNapiLoader {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { inner: NapiLoader::new() }
    }

    #[wasm_bindgen]
    pub fn is_napi_module(name: &str) -> bool {
        NapiLoader::is_napi_module(name)
    }

    #[wasm_bindgen]
    pub fn napi_version(&self) -> u32 {
        self.inner.napi_version()
    }

    #[wasm_bindgen]
    pub fn list_loaded(&self) -> Vec<JsValue> {
        self.inner.list_loaded().into_iter().map(JsValue::from).collect()
    }

    #[wasm_bindgen]
    pub fn is_loaded(&self, name: &str) -> bool {
        self.inner.is_loaded(name)
    }

    #[wasm_bindgen]
    pub fn unload(&mut self, name: &str) -> bool {
        self.inner.unload(name)
    }

    #[wasm_bindgen]
    pub fn clear(&mut self) {
        self.inner.clear();
    }
}
"""

def rust_client_napi():
    return """use crate::{NapiLoader, NapiModule};

pub struct NapiClient {
    loader: NapiLoader,
}

impl NapiClient {
    pub fn new() -> Self {
        Self { loader: NapiLoader::new() }
    }

    pub fn load(&mut self, name: &str) -> anyhow::Result<&NapiModule> {
        self.loader.load(name)
    }

    pub fn list(&self) -> Vec<String> {
        self.loader.list_loaded()
    }

    pub fn unload(&mut self, name: &str) -> bool {
        self.loader.unload(name)
    }

    pub fn clear(&mut self) {
        self.loader.clear();
    }

    pub fn version(&self) -> u32 {
        self.loader.napi_version()
    }
}
"""

def rust_server_napi():
    return """use crate::NapiLoader;
use std::sync::{Arc, Mutex};

pub struct NapiServer {
    loader: Arc<Mutex<NapiLoader>>,
}

impl NapiServer {
    pub fn new() -> Self {
        Self { loader: Arc::new(Mutex::new(NapiLoader::new())) }
    }

    pub fn load(&self, name: &str) -> anyhow::Result<()> {
        self.loader.lock().unwrap().load(name)?;
        Ok(())
    }

    pub fn list(&self) -> Vec<String> {
        self.loader.lock().unwrap().list_loaded()
    }

    pub fn unload(&self, name: &str) -> bool {
        self.loader.lock().unwrap().unload(name)
    }

    pub fn clear(&self) {
        self.loader.lock().unwrap().clear();
    }

    pub fn loader_ref(&self) -> Arc<Mutex<NapiLoader>> {
        self.loader.clone()
    }
}
"""

def rust_test_utils_napi():
    return """#![cfg(test)]
use crate::NapiLoader;

pub fn create_test_loader() -> NapiLoader {
    NapiLoader::new()
}

pub fn create_mock_module(name: &str) -> crate::NapiModule {
    use std::collections::HashMap;
    crate::NapiModule {
        name: name.to_string(),
        exports: HashMap::new(),
    }
}

#[test]
fn test_test_utils() {
    let mut loader = create_test_loader();
    assert!(loader.list_loaded().is_empty());
    let _module = create_mock_module("test");
}
"""

def rust_benchmark_napi():
    return """#![cfg(test)]
use crate::NapiLoader;

#[test]
fn bench_napi_loader() {
    let loader = NapiLoader::new();
    let start = std::time::Instant::now();
    for _ in 0..1000 {
        let _ = loader.napi_version();
    }
    let elapsed = start.elapsed();
    println!("napi_version() x1000: {:?}", elapsed);
}

#[test]
fn bench_is_napi_module() {
    let start = std::time::Instant::now();
    for _ in 0..10000 {
        let _ = NapiLoader::is_napi_module("addon.node");
    }
    let elapsed = start.elapsed();
    println!("is_napi_module() x10000: {:?}", elapsed);
}
"""

def rust_lib_napi():
    return r"""pub mod types;
pub mod errors;
pub mod builder;
pub mod config;
pub mod ffi;
pub mod wasm;
pub mod client;
pub mod server;
pub mod test_utils;
pub mod benchmark;

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct NapiModule {
    pub name: String,
    pub exports: HashMap<String, serde_json::Value>,
}

pub struct NapiLoader {
    loaded_modules: HashMap<String, NapiModule>,
    config: Option<config::NapiConfig>,
}

impl NapiLoader {
    pub fn new() -> Self {
        Self { loaded_modules: HashMap::new(), config: None }
    }

    pub fn with_config(config: config::NapiConfig) -> Self {
        Self { loaded_modules: HashMap::new(), config: Some(config) }
    }

    pub fn load(&mut self, name: &str) -> anyhow::Result<&NapiModule> {
        if self.loaded_modules.contains_key(name) {
            return Ok(self.loaded_modules.get(name).unwrap());
        }
        let module = self.load_native_module(name)?;
        self.loaded_modules.insert(name.to_string(), module);
        Ok(self.loaded_modules.get(name).unwrap())
    }

    fn load_native_module(&self, name: &str) -> anyhow::Result<NapiModule> {
        let node_modules_path = std::env::current_dir()
            .unwrap_or_default()
            .join("node_modules")
            .join(name);
        let binding_path = if cfg!(target_os = "linux") {
            node_modules_path.join(&format!("{name}.linux-x64-gnu.node"))
        } else if cfg!(target_os = "macos") {
            node_modules_path.join(&format!("{name}.darwin-x64.node"))
        } else {
            node_modules_path.join(&format!("{name}.win32-x64-msvc.node"))
        };
        if !binding_path.exists() {
            anyhow::bail!("N-API module '{name}' not found at: {}", binding_path.display());
        }
        Ok(NapiModule { name: name.to_string(), exports: HashMap::new() })
    }

    pub fn list_loaded(&self) -> Vec<String> {
        self.loaded_modules.keys().cloned().collect()
    }

    pub fn is_loaded(&self, name: &str) -> bool { self.loaded_modules.contains_key(name) }
    pub fn unload(&mut self, name: &str) -> bool { self.loaded_modules.remove(name).is_some() }
    pub fn clear(&mut self) { self.loaded_modules.clear(); }
    pub fn symbol_count(&self) -> usize { self.loaded_modules.values().map(|m| m.exports.len()).sum() }
    pub fn napi_version(&self) -> u32 { 9 }
    pub fn is_napi_module(name: &str) -> bool { name.ends_with(".node") }
}

impl Default for NapiLoader { fn default() -> Self { Self::new() } }
"""

# ============================================================
# TypeScript BINDINGS
# ============================================================

def ts_types_napi():
    return """export interface NapiModule {
  name: string;
  exports: Record<string, unknown>;
}

export interface NapiLoaderConfig {
  modulePaths: string[];
  cacheEnabled: boolean;
}

export interface NapiVersion {
  major: number;
  minor: number;
  patch: number;
}

export interface NapiModuleInfo {
  name: string;
  loaded: boolean;
  symbolCount: number;
}
"""

def ts_client_napi():
    return """import { NapiModule, NapiModuleInfo } from './types';

export class NapiClient {
  private modules: Map<string, NapiModule> = new Map();

  load(name: string): NapiModule {
    const existing = this.modules.get(name);
    if (existing) return existing;
    const mod: NapiModule = { name, exports: {} };
    this.modules.set(name, mod);
    return mod;
  }

  list(): string[] {
    return Array.from(this.modules.keys());
  }

  unload(name: string): boolean {
    return this.modules.delete(name);
  }

  clear(): void {
    this.modules.clear();
  }

  isLoaded(name: string): boolean {
    return this.modules.has(name);
  }

  info(): NapiModuleInfo[] {
    return Array.from(this.modules.entries()).map(([name, mod]) => ({
      name,
      loaded: true,
      symbolCount: Object.keys(mod.exports).length,
    }));
  }

  version(): number {
    return 9;
  }

  static isNapiModule(name: string): boolean {
    return name.endsWith('.node');
  }
}
"""

def ts_errors_napi():
    return """export class NapiError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'NapiError';
  }
}

export class ModuleNotFoundError extends NapiError {
  constructor(moduleName: string) {
    super(`N-API module '${moduleName}' not found`);
    this.name = 'ModuleNotFoundError';
  }
}

export class LoadFailedError extends NapiError {
  constructor(moduleName: string, reason: string) {
    super(`Failed to load '${moduleName}': ${reason}`);
    this.name = 'LoadFailedError';
  }
}

export class VersionMismatchError extends NapiError {
  constructor(expected: number, got: number) {
    super(`N-API version mismatch: expected ${expected}, got ${got}`);
    this.name = 'VersionMismatchError';
  }
}
"""

def ts_config_napi():
    return """import { NapiLoaderConfig } from './types';

export interface NapiConfig {
  loader: NapiLoaderConfig;
  napiVersion: number;
}

export const DEFAULT_NAPI_CONFIG: NapiConfig = {
  loader: {
    modulePaths: ['node_modules'],
    cacheEnabled: true,
  },
  napiVersion: 9,
};

export function createNapiConfig(
  overrides?: Partial<NapiConfig>
): NapiConfig {
  return { ...DEFAULT_NAPI_CONFIG, ...overrides };
}
"""

def ts_utils_napi():
    return """import { NapiModule, NapiLoaderConfig } from './types';

export function detectModulePath(name: string): string {
  const platform =
    process.platform === 'linux'
      ? 'linux-x64-gnu'
      : process.platform === 'darwin'
        ? 'darwin-x64'
        : 'win32-x64-msvc';
  const cwd = process.cwd();
  return `${cwd}/node_modules/${name}/${name}.${platform}.node`;
}

export function mergeLoaderConfigs(
  base: NapiLoaderConfig,
  override: Partial<NapiLoaderConfig>
): NapiLoaderConfig {
  return {
    ...base,
    ...override,
    modulePaths: [...base.modulePaths, ...(override.modulePaths || [])],
  };
}
"""

def ts_index_napi():
    return """export { NapiClient } from './client';
export { NapiError, ModuleNotFoundError, LoadFailedError, VersionMismatchError } from './errors';
export { createNapiConfig, DEFAULT_NAPI_CONFIG, NapiConfig } from './config';
export { NapiModule, NapiLoaderConfig, NapiVersion, NapiModuleInfo } from './types';
export { detectModulePath, mergeLoaderConfigs } from './utils';
"""

def ts_test_napi():
    return """import { NapiClient } from './client';
import { NapiModule } from './types';

export function createTestClient(): NapiClient {
  return new NapiClient();
}

export function createMockModule(name: string): NapiModule {
  return { name, exports: {} };
}

export function assertModuleLoaded(client: NapiClient, name: string): void {
  if (!client.isLoaded(name)) {
    throw new Error(`Expected module '${name}' to be loaded`);
  }
}

export function assertModuleNotLoaded(client: NapiClient, name: string): void {
  if (client.isLoaded(name)) {
    throw new Error(`Expected module '${name}' to not be loaded`);
  }
}
"""

def ts_benchmark_napi():
    return """import { NapiClient } from './client';

export function benchLoad(iterations: number): number {
  const client = new NapiClient();
  const start = process.hrtime.bigint();
  for (let i = 0; i < iterations; i++) {
    client.version();
  }
  const end = process.hrtime.bigint();
  return Number(end - start) / 1e6;
}

export function benchIsNapiModule(iterations: number): number {
  const start = process.hrtime.bigint();
  for (let i = 0; i < iterations; i++) {
    NapiClient.isNapiModule('addon.node');
  }
  const end = process.hrtime.bigint();
  return Number(end - start) / 1e6;
}

export function runBenchmarks(): void {
  console.log(`load x1000: ${benchLoad(1000).toFixed(2)}ms`);
  console.log(`isNapiModule x10000: ${benchIsNapiModule(10000).toFixed(2)}ms`);
}
"""

def ts_fixtures_napi():
    return """import { NapiModule, NapiLoaderConfig } from './types';

export const TEST_MODULE_NAMES = ['test-addon', 'sample.node', 'native-bindings'];

export const MOCK_MODULES: Record<string, NapiModule> = {
  'test-addon': { name: 'test-addon', exports: { hello: 'world' } },
  'sample.node': { name: 'sample.node', exports: { add: (a: number, b: number) => a + b } },
};

export const TEST_CONFIGS: NapiLoaderConfig[] = [
  { modulePaths: ['node_modules'], cacheEnabled: true },
  { modulePaths: ['node_modules', './lib'], cacheEnabled: false },
];

export function getMockModule(name: string): NapiModule | undefined {
  return MOCK_MODULES[name];
}
"""

def ts_declaration_napi():
    return """declare module '*.node' {
  const exports: Record<string, unknown>;
  export default exports;
}

declare namespace NodeJS {
  interface Process {
    platform: 'linux' | 'darwin' | 'win32';
  }
}
"""

# ============================================================
# C++ BINDINGS
# ============================================================

def cpp_types_hpp_napi():
    return """#pragma once
#include <string>
#include <unordered_map>
#include <vector>
#include <any>

namespace klyron_napi {

struct NapiModule {
    std::string name;
    std::unordered_map<std::string, std::any> exports;
};

struct NapiLoaderConfig {
    std::vector<std::string> module_paths;
    bool cache_enabled = true;
};

struct NapiVersion {
    uint32_t major = 9;
    uint32_t minor = 0;
    uint32_t patch = 0;
};

} // namespace klyron_napi
"""

def cpp_types_cpp_napi():
    return """#include "types.hpp"

namespace klyron_napi {
    // Types are header-only
}
"""

def cpp_api_hpp_napi():
    return """#pragma once
#include "types.hpp"
#include <memory>

namespace klyron_napi {

class NapiLoader {
public:
    NapiLoader();
    explicit NapiLoader(const NapiLoaderConfig& config);

    std::shared_ptr<NapiModule> load(const std::string& name);
    std::vector<std::string> list_loaded() const;
    bool is_loaded(const std::string& name) const;
    bool unload(const std::string& name);
    void clear();
    size_t symbol_count() const;
    uint32_t napi_version() const;
    static bool is_napi_module(const std::string& name);

private:
    std::unordered_map<std::string, std::shared_ptr<NapiModule>> loaded_modules_;
    NapiLoaderConfig config_;
};

class NapiClient {
public:
    NapiClient();
    std::shared_ptr<NapiModule> load(const std::string& name);
    std::vector<std::string> list() const;
    bool unload(const std::string& name);
    void clear();
    uint32_t version() const;

private:
    NapiLoader loader_;
};

} // namespace klyron_napi
"""

def cpp_api_cpp_napi():
    return """#include "api.hpp"
#include <stdexcept>

namespace klyron_napi {

NapiLoader::NapiLoader() : config_() {}

NapiLoader::NapiLoader(const NapiLoaderConfig& config) : config_(config) {}

std::shared_ptr<NapiModule> NapiLoader::load(const std::string& name) {
    auto it = loaded_modules_.find(name);
    if (it != loaded_modules_.end()) return it->second;
    auto module = std::make_shared<NapiModule>();
    module->name = name;
    loaded_modules_[name] = module;
    return module;
}

std::vector<std::string> NapiLoader::list_loaded() const {
    std::vector<std::string> keys;
    for (const auto& [key, _] : loaded_modules_) keys.push_back(key);
    return keys;
}

bool NapiLoader::is_loaded(const std::string& name) const {
    return loaded_modules_.find(name) != loaded_modules_.end();
}

bool NapiLoader::unload(const std::string& name) {
    return loaded_modules_.erase(name) > 0;
}

void NapiLoader::clear() { loaded_modules_.clear(); }

size_t NapiLoader::symbol_count() const {
    size_t count = 0;
    for (const auto& [_, mod] : loaded_modules_) count += mod->exports.size();
    return count;
}

uint32_t NapiLoader::napi_version() const { return 9; }

bool NapiLoader::is_napi_module(const std::string& name) {
    return name.size() >= 5 && name.substr(name.size() - 5) == ".node";
}

NapiClient::NapiClient() : loader_() {}

std::shared_ptr<NapiModule> NapiClient::load(const std::string& name) {
    return loader_.load(name);
}

std::vector<std::string> NapiClient::list() const { return loader_.list_loaded(); }

bool NapiClient::unload(const std::string& name) { return loader_.unload(name); }

void NapiClient::clear() { loader_.clear(); }

uint32_t NapiClient::version() const { return loader_.napi_version(); }

} // namespace klyron_napi
"""

def cpp_config_hpp_napi():
    return """#pragma once
#include "types.hpp"

namespace klyron_napi {

class NapiConfig {
public:
    NapiConfig();
    explicit NapiConfig(const NapiLoaderConfig& loader_config);
    NapiConfig with_loader(const NapiLoaderConfig& loader_config) const;
    NapiLoaderConfig loader_config() const;
    uint32_t napi_version() const;
    static NapiConfig defaults();

private:
    NapiLoaderConfig loader_config_;
    uint32_t napi_version_ = 9;
};

} // namespace klyron_napi
"""

def cpp_config_cpp_napi():
    return """#include "config.hpp"

namespace klyron_napi {

NapiConfig::NapiConfig() : loader_config_(), napi_version_(9) {}

NapiConfig::NapiConfig(const NapiLoaderConfig& loader_config)
    : loader_config_(loader_config), napi_version_(9) {}

NapiConfig NapiConfig::with_loader(const NapiLoaderConfig& loader_config) const {
    return NapiConfig(loader_config);
}

NapiLoaderConfig NapiConfig::loader_config() const { return loader_config_; }

uint32_t NapiConfig::napi_version() const { return napi_version_; }

NapiConfig NapiConfig::defaults() { return NapiConfig(); }

} // namespace klyron_napi
"""

def cpp_errors_hpp_napi():
    return """#pragma once
#include <stdexcept>
#include <string>

namespace klyron_napi {

class NapiException : public std::runtime_error {
public:
    explicit NapiException(const std::string& message) : std::runtime_error(message) {}
};

class ModuleNotFoundError : public NapiException {
public:
    explicit ModuleNotFoundError(const std::string& name)
        : NapiException("N-API module '" + name + "' not found") {}
};

class LoadFailedError : public NapiException {
public:
    LoadFailedError(const std::string& name, const std::string& reason)
        : NapiException("Failed to load '" + name + "': " + reason) {}
};

class VersionMismatchError : public NapiException {
public:
    VersionMismatchError(uint32_t expected, uint32_t got)
        : NapiException("Version mismatch: expected " + std::to_string(expected)
                        + ", got " + std::to_string(got)) {}
};

} // namespace klyron_napi
"""

def cpp_errors_cpp_napi():
    return """#include "errors.hpp"
// Error classes are header-only
"""

def cpp_builder_hpp_napi():
    return """#pragma once
#include "types.hpp"
#include "api.hpp"
#include <string>

namespace klyron_napi {

class NapiLoaderBuilder {
public:
    NapiLoaderBuilder();
    NapiLoaderBuilder& module_path(const std::string& path);
    NapiLoaderBuilder& cache_enabled(bool enabled);
    NapiLoader build();

private:
    NapiLoaderConfig config_;
};

} // namespace klyron_napi
"""

def cpp_builder_cpp_napi():
    return """#include "builder.hpp"

namespace klyron_napi {

NapiLoaderBuilder::NapiLoaderBuilder() : config_() {}

NapiLoaderBuilder& NapiLoaderBuilder::module_path(const std::string& path) {
    config_.module_paths.push_back(path);
    return *this;
}

NapiLoaderBuilder& NapiLoaderBuilder::cache_enabled(bool enabled) {
    config_.cache_enabled = enabled;
    return *this;
}

NapiLoader NapiLoaderBuilder::build() {
    return NapiLoader(config_);
}

} // namespace klyron_napi
"""

def cpp_ffi_hpp_napi():
    return """#pragma once
#include <cstdint>

extern "C" {

struct napi_module_t {
    const char* name;
    const char* exports;
    uint32_t napi_version;
};

napi_module_t* napi_load_module(const char* name);
void napi_free_module(napi_module_t* module);

}
"""

def cpp_ffi_cpp_napi():
    return """#include "ffi.hpp"
#include "api.hpp"
#include <cstring>
#include <memory>

extern "C" {

napi_module_t* napi_load_module(const char* name) {
    if (!name) return nullptr;
    auto loader = std::make_unique<klyron_napi::NapiLoader>();
    try {
        auto mod = loader->load(name);
        auto c_name = strdup(mod->name.c_str());
        auto c_exports = strdup("{}");
        auto result = new napi_module_t{c_name, c_exports, 9};
        return result;
    } catch (...) {
        return nullptr;
    }
}

void napi_free_module(napi_module_t* module) {
    if (module) {
        free(const_cast<char*>(module->name));
        free(const_cast<char*>(module->exports));
        delete module;
    }
}

}
"""

# ============================================================
# C BINDINGS
# ============================================================

def c_header_h_napi():
    return """#ifndef KLYRON_NAPI_H
#define KLYRON_NAPI_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    char* name;
    char* exports;
    uint32_t napi_version;
} klyron_napi_module_t;

typedef struct klyron_napi_loader_t klyron_napi_loader_t;

klyron_napi_loader_t* klyron_napi_loader_new(void);
klyron_napi_loader_t* klyron_napi_loader_with_config(const char* config_json);
void klyron_napi_loader_free(klyron_napi_loader_t* loader);

klyron_napi_module_t* klyron_napi_load(klyron_napi_loader_t* loader, const char* name);
void klyron_napi_module_free(klyron_napi_module_t* module);

char** klyron_napi_list_loaded(klyron_napi_loader_t* loader, size_t* count);
bool klyron_napi_is_loaded(klyron_napi_loader_t* loader, const char* name);
bool klyron_napi_unload(klyron_napi_loader_t* loader, const char* name);
void klyron_napi_clear(klyron_napi_loader_t* loader);
uint32_t klyron_napi_version(klyron_napi_loader_t* loader);
bool klyron_napi_is_napi_module(const char* name);

#ifdef __cplusplus
}
#endif

#endif /* KLYRON_NAPI_H */
"""

def c_impl_c_napi():
    return """#include "klyron_napi.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

typedef struct klyron_napi_loader_t {
    char** modules;
    size_t count;
    size_t capacity;
} klyron_napi_loader_t;

klyron_napi_loader_t* klyron_napi_loader_new(void) {
    klyron_napi_loader_t* loader = calloc(1, sizeof(klyron_napi_loader_t));
    if (loader) {
        loader->capacity = 16;
        loader->modules = calloc(loader->capacity, sizeof(char*));
    }
    return loader;
}

klyron_napi_loader_t* klyron_napi_loader_with_config(const char* config_json) {
    (void)config_json;
    return klyron_napi_loader_new();
}

void klyron_napi_loader_free(klyron_napi_loader_t* loader) {
    if (loader) {
        for (size_t i = 0; i < loader->count; i++) free(loader->modules[i]);
        free(loader->modules);
        free(loader);
    }
}

klyron_napi_module_t* klyron_napi_load(klyron_napi_loader_t* loader, const char* name) {
    if (!loader || !name) return NULL;
    for (size_t i = 0; i < loader->count; i++) {
        if (strcmp(loader->modules[i], name) == 0) {
            klyron_napi_module_t* mod = malloc(sizeof(klyron_napi_module_t));
            mod->name = strdup(name);
            mod->exports = strdup("{}");
            mod->napi_version = 9;
            return mod;
        }
    }
    if (loader->count >= loader->capacity) {
        loader->capacity *= 2;
        loader->modules = realloc(loader->modules, loader->capacity * sizeof(char*));
    }
    loader->modules[loader->count++] = strdup(name);
    klyron_napi_module_t* mod = malloc(sizeof(klyron_napi_module_t));
    mod->name = strdup(name);
    mod->exports = strdup("{}");
    mod->napi_version = 9;
    return mod;
}

void klyron_napi_module_free(klyron_napi_module_t* mod) {
    if (mod) { free(mod->name); free(mod->exports); free(mod); }
}

char** klyron_napi_list_loaded(klyron_napi_loader_t* loader, size_t* count) {
    if (!loader || !count) return NULL;
    *count = loader->count;
    char** list = calloc(loader->count, sizeof(char*));
    for (size_t i = 0; i < loader->count; i++) list[i] = strdup(loader->modules[i]);
    return list;
}

bool klyron_napi_is_loaded(klyron_napi_loader_t* loader, const char* name) {
    if (!loader || !name) return false;
    for (size_t i = 0; i < loader->count; i++)
        if (strcmp(loader->modules[i], name) == 0) return true;
    return false;
}

bool klyron_napi_unload(klyron_napi_loader_t* loader, const char* name) {
    if (!loader || !name) return false;
    for (size_t i = 0; i < loader->count; i++) {
        if (strcmp(loader->modules[i], name) == 0) {
            free(loader->modules[i]);
            loader->modules[i] = loader->modules[--loader->count];
            return true;
        }
    }
    return false;
}

void klyron_napi_clear(klyron_napi_loader_t* loader) {
    if (loader) {
        for (size_t i = 0; i < loader->count; i++) free(loader->modules[i]);
        loader->count = 0;
    }
}

uint32_t klyron_napi_version(klyron_napi_loader_t* loader) {
    (void)loader;
    return 9;
}

bool klyron_napi_is_napi_module(const char* name) {
    if (!name) return false;
    size_t len = strlen(name);
    return len >= 5 && strcmp(name + len - 5, ".node") == 0;
}
"""

def c_types_h_napi():
    return """#ifndef KLYRON_NAPI_TYPES_H
#define KLYRON_NAPI_TYPES_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    char* name;
    char* exports;
    uint32_t napi_version;
} klyron_napi_module_t;

typedef struct {
    char** paths;
    size_t path_count;
    bool cache_enabled;
} klyron_napi_config_t;

typedef struct {
    uint32_t major;
    uint32_t minor;
    uint32_t patch;
} klyron_napi_version_t;

#ifdef __cplusplus
}
#endif

#endif /* KLYRON_NAPI_TYPES_H */
"""

def c_errors_h_napi():
    return """#ifndef KLYRON_NAPI_ERRORS_H
#define KLYRON_NAPI_ERRORS_H

#ifdef __cplusplus
extern "C" {
#endif

typedef enum {
    KLYRON_NAPI_OK = 0,
    KLYRON_NAPI_ERR_MODULE_NOT_FOUND = -1,
    KLYRON_NAPI_ERR_LOAD_FAILED = -2,
    KLYRON_NAPI_ERR_VERSION_MISMATCH = -3,
    KLYRON_NAPI_ERR_INVALID_ARGUMENT = -4,
} klyron_napi_error_t;

const char* klyron_napi_error_string(klyron_napi_error_t err);

#ifdef __cplusplus
}
#endif

#endif /* KLYRON_NAPI_ERRORS_H */
"""

def c_errors_c_napi():
    return """#include "errors.h"

const char* klyron_napi_error_string(klyron_napi_error_t err) {
    switch (err) {
        case KLYRON_NAPI_OK: return "success";
        case KLYRON_NAPI_ERR_MODULE_NOT_FOUND: return "module not found";
        case KLYRON_NAPI_ERR_LOAD_FAILED: return "load failed";
        case KLYRON_NAPI_ERR_VERSION_MISMATCH: return "version mismatch";
        case KLYRON_NAPI_ERR_INVALID_ARGUMENT: return "invalid argument";
        default: return "unknown error";
    }
}
"""

def c_config_h_napi():
    return """#ifndef KLYRON_NAPI_CONFIG_H
#define KLYRON_NAPI_CONFIG_H

#include "types.h"
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

klyron_napi_config_t* klyron_napi_config_default(void);
void klyron_napi_config_free(klyron_napi_config_t* config);

#ifdef __cplusplus
}
#endif

#endif /* KLYRON_NAPI_CONFIG_H */
"""

def c_config_c_napi():
    return """#include "config.h"
#include <stdlib.h>
#include <string.h>

klyron_napi_config_t* klyron_napi_config_default(void) {
    klyron_napi_config_t* config = calloc(1, sizeof(klyron_napi_config_t));
    if (config) {
        config->cache_enabled = true;
        config->path_count = 1;
        config->paths = calloc(1, sizeof(char*));
        config->paths[0] = strdup("node_modules");
    }
    return config;
}

void klyron_napi_config_free(klyron_napi_config_t* config) {
    if (config) {
        for (size_t i = 0; i < config->path_count; i++) free(config->paths[i]);
        free(config->paths);
        free(config);
    }
}
"""

def c_utils_h_napi():
    return """#ifndef KLYRON_NAPI_UTILS_H
#define KLYRON_NAPI_UTILS_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

void klyron_napi_free_strings(char** strings, size_t count);

#ifdef __cplusplus
}
#endif

#endif /* KLYRON_NAPI_UTILS_H */
"""

def c_utils_c_napi():
    return """#include "utils.h"
#include <stdlib.h>

void klyron_napi_free_strings(char** strings, size_t count) {
    if (strings) {
        for (size_t i = 0; i < count; i++) free(strings[i]);
        free(strings);
    }
}
"""

# ============================================================
# BASH SCRIPTS
# ============================================================

def bash_build():
    return """#!/usr/bin/env bash
set -euo pipefail

CRATE_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$CRATE_DIR"

echo "==> Building crate..."
cargo build --release 2>&1
echo "==> Build complete"
"""

def bash_test():
    return """#!/usr/bin/env bash
set -euo pipefail

CRATE_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$CRATE_DIR"

echo "==> Running tests..."
cargo test 2>&1
echo "==> Tests complete"
"""

def bash_bench():
    return """#!/usr/bin/env bash
set -euo pipefail

CRATE_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$CRATE_DIR"

echo "==> Running benchmarks..."
cargo bench 2>&1 || cargo test -- --bench 2>&1 || echo "No benchmarks found"
echo "==> Benchmarks complete"
"""

def bash_dev():
    return """#!/usr/bin/env bash
set -euo pipefail

CRATE_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$CRATE_DIR"

case "${1:-build}" in
    build)
        cargo build
        ;;
    watch)
        cargo watch -x build
        ;;
    test)
        cargo test
        ;;
    doc)
        cargo doc --open
        ;;
    check)
        cargo check
        ;;
    *)
        echo "Usage: $0 {build|watch|test|doc|check}"
        exit 1
        ;;
esac
"""

def bash_clean():
    return """#!/usr/bin/env bash
set -euo pipefail

CRATE_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$CRATE_DIR"

echo "==> Cleaning crate artifacts..."
cargo clean 2>&1
rm -rf target/ 2>/dev/null || true
echo "==> Clean complete"
"""


# ============================================================
# GENERATE ALL FILES
# ============================================================

def generate_rust_files(crate_name, types_fn, errors_fn, builder_fn, config_fn, ffi_fn, wasm_fn, client_fn, server_fn, test_utils_fn, lib_fn):
    crate_bind = f"{BASE}/{crate_name}/bindings/rust"
    ensure_dir(crate_bind)
    write_file(f"{crate_bind}/types.rs", types_fn())
    write_file(f"{crate_bind}/errors.rs", errors_fn())
    write_file(f"{crate_bind}/builder.rs", builder_fn())
    write_file(f"{crate_bind}/config.rs", config_fn())
    write_file(f"{crate_bind}/ffi.rs", ffi_fn())
    write_file(f"{crate_bind}/wasm.rs", wasm_fn())
    write_file(f"{crate_bind}/client.rs", client_fn())
    write_file(f"{crate_bind}/server.rs", server_fn())
    write_file(f"{crate_bind}/test_utils.rs", test_utils_fn())
    write_file(f"{crate_bind}/benchmark.rs", lib_fn())  # reuse as benchmark

def generate_ts_files(crate_name, types_fn, client_fn, errors_fn, config_fn, utils_fn, index_fn, test_fn, benchmark_fn, fixtures_fn, declaration_fn):
    crate_bind = f"{BASE}/{crate_name}/bindings/ts"
    ensure_dir(crate_bind)
    write_file(f"{crate_bind}/types.ts", types_fn())
    write_file(f"{crate_bind}/client.ts", client_fn())
    write_file(f"{crate_bind}/errors.ts", errors_fn())
    write_file(f"{crate_bind}/config.ts", config_fn())
    write_file(f"{crate_bind}/utils.ts", utils_fn())
    write_file(f"{crate_bind}/index.ts", index_fn())
    write_file(f"{crate_bind}/test.ts", test_fn())
    write_file(f"{crate_bind}/benchmark.ts", benchmark_fn())
    write_file(f"{crate_bind}/fixtures.ts", fixtures_fn())
    write_file(f"{crate_bind}/declaration.d.ts", declaration_fn())

def generate_cpp_files(crate_name, types_hpp_fn, types_cpp_fn, api_hpp_fn, api_cpp_fn, config_hpp_fn, config_cpp_fn, errors_hpp_fn, errors_cpp_fn, builder_hpp_fn, builder_cpp_fn, ffi_hpp_fn, ffi_cpp_fn):
    crate_bind = f"{BASE}/{crate_name}/bindings/cpp"
    ensure_dir(crate_bind)
    write_file(f"{crate_bind}/types.hpp", types_hpp_fn())
    write_file(f"{crate_bind}/types.cpp", types_cpp_fn())
    write_file(f"{crate_bind}/api.hpp", api_hpp_fn())
    write_file(f"{crate_bind}/api.cpp", api_cpp_fn())
    write_file(f"{crate_bind}/config.hpp", config_hpp_fn())
    write_file(f"{crate_bind}/config.cpp", config_cpp_fn())
    write_file(f"{crate_bind}/errors.hpp", errors_hpp_fn())
    write_file(f"{crate_bind}/errors.cpp", errors_cpp_fn())
    write_file(f"{crate_bind}/builder.hpp", builder_hpp_fn())
    write_file(f"{crate_bind}/builder.cpp", builder_cpp_fn())
    write_file(f"{crate_bind}/ffi.hpp", ffi_hpp_fn())
    write_file(f"{crate_bind}/ffi.cpp", ffi_cpp_fn())

def generate_c_files(crate_name, header_fn, impl_fn, types_h_fn, errors_h_fn, errors_c_fn, config_h_fn, config_c_fn, utils_h_fn, utils_c_fn):
    crate_bind = f"{BASE}/{crate_name}/bindings/c"
    ensure_dir(crate_bind)
    write_file(f"{crate_bind}/klyron_{crate_name.replace('klyron_', '')}.h", header_fn())
    write_file(f"{crate_bind}/klyron_{crate_name.replace('klyron_', '')}.c", impl_fn())
    write_file(f"{crate_bind}/types.h", types_h_fn())
    write_file(f"{crate_bind}/errors.h", errors_h_fn())
    write_file(f"{crate_bind}/errors.c", errors_c_fn())
    write_file(f"{crate_bind}/config.h", config_h_fn())
    write_file(f"{crate_bind}/config.c", config_c_fn())
    write_file(f"{crate_bind}/utils.h", utils_h_fn())
    write_file(f"{crate_bind}/utils.c", utils_c_fn())

def generate_bash_files(crate_name):
    crate_bind = f"{BASE}/{crate_name}/bindings/bash"
    ensure_dir(crate_bind)
    write_file(f"{crate_bind}/build.sh", bash_build())
    write_file(f"{crate_bind}/test.sh", bash_test())
    write_file(f"{crate_bind}/bench.sh", bash_bench())
    write_file(f"{crate_bind}/dev.sh", bash_dev())
    write_file(f"{crate_bind}/clean.sh", bash_clean())
    os.chmod(f"{crate_bind}/build.sh", 0o755)
    os.chmod(f"{crate_bind}/test.sh", 0o755)
    os.chmod(f"{crate_bind}/bench.sh", 0o755)
    os.chmod(f"{crate_bind}/dev.sh", 0o755)
    os.chmod(f"{crate_bind}/clean.sh", 0o755)


# ============================================================
# PER-CRATE GENERATORS
# ============================================================

# --- klyron_napi ---
generate_rust_files("klyron_napi", rust_types_napi, rust_errors_napi, rust_builder_napi, rust_config_napi, rust_ffi_napi, rust_wasm_napi, rust_client_napi, rust_server_napi, rust_test_utils_napi, rust_benchmark_napi)
generate_ts_files("klyron_napi", ts_types_napi, ts_client_napi, ts_errors_napi, ts_config_napi, ts_utils_napi, ts_index_napi, ts_test_napi, ts_benchmark_napi, ts_fixtures_napi, ts_declaration_napi)
generate_cpp_files("klyron_napi", cpp_types_hpp_napi, cpp_types_cpp_napi, cpp_api_hpp_napi, cpp_api_cpp_napi, cpp_config_hpp_napi, cpp_config_cpp_napi, cpp_errors_hpp_napi, cpp_errors_cpp_napi, cpp_builder_hpp_napi, cpp_builder_cpp_napi, cpp_ffi_hpp_napi, cpp_ffi_cpp_napi)
generate_c_files("klyron_napi", c_header_h_napi, c_impl_c_napi, c_types_h_napi, c_errors_h_napi, c_errors_c_napi, c_config_h_napi, c_config_c_napi, c_utils_h_napi, c_utils_c_napi)
generate_bash_files("klyron_napi")

# ============================================================
# GENERATE OTHER CRATES USING CRATE-SPECIFIC CONTENT
# ============================================================

def rust_types_pm():
    return """use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PackageManager {
    Npm, Yarn, Pnpm, Bun, Composer, Cargo, Go, Pip, Gem,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub manager: PackageManager,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallOptions {
    pub dev: bool,
    pub global: bool,
    pub frozen_lockfile: bool,
}
"""

def rust_errors_pm():
    return """use thiserror::Error;

#[derive(Error, Debug)]
pub enum PmError {
    #[error("package manager not found")]
    NotFound,
    #[error("install failed: {0}")]
    InstallFailed(String),
    #[error("unsupported package manager: {0:?}")]
    Unsupported(PackageManager),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
use crate::types::PackageManager;
"""

def rust_builder_pm():
    return """use crate::types::{InstallOptions, PackageManager};

#[derive(Debug, Clone)]
pub struct PmBuilder {
    manager: Option<PackageManager>,
    path: std::path::PathBuf,
    options: InstallOptions,
}

impl PmBuilder {
    pub fn new(path: &std::path::Path) -> Self {
        Self { manager: None, path: path.to_path_buf(), options: InstallOptions { dev: false, global: false, frozen_lockfile: false } }
    }

    pub fn manager(mut self, m: PackageManager) -> Self { self.manager = Some(m); self }
    pub fn dev(mut self, v: bool) -> Self { self.options.dev = v; self }
    pub fn frozen_lockfile(mut self, v: bool) -> Self { self.options.frozen_lockfile = v; self }
    pub fn build(self) -> super::PackageManagerClient {
        let pm = self.manager.unwrap_or_else(|| super::detect(&self.path));
        super::PackageManagerClient::new(pm, self.path)
    }
}
"""

def rust_config_pm():
    return """use crate::types::PackageManager;

#[derive(Debug, Clone)]
pub struct PmConfig {
    pub default_manager: PackageManager,
    pub lockfile_check: bool,
    pub auto_install: bool,
}

impl Default for PmConfig {
    fn default() -> Self {
        Self { default_manager: PackageManager::Npm, lockfile_check: true, auto_install: false }
    }
}
"""

def rust_ffi_pm():
    return """use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn pm_detect(path: *const c_char) -> i32 {
    let path_str = unsafe { CStr::from_ptr(path) }.to_string_lossy().into_owned();
    let pm = super::detect(std::path::Path::new(&path_str));
    pm as i32
}

#[no_mangle]
pub extern "C" fn pm_install_cmd(pm: i32) -> *mut c_char {
    let manager = match pm {
        0 => super::PackageManager::Npm,
        1 => super::PackageManager::Yarn,
        2 => super::PackageManager::Pnpm,
        3 => super::PackageManager::Bun,
        4 => super::PackageManager::Composer,
        5 => super::PackageManager::Cargo,
        6 => super::PackageManager::Go,
        7 => super::PackageManager::Pip,
        8 => super::PackageManager::Gem,
        _ => super::PackageManager::Npm,
    };
    CString::new(super::install_cmd(manager)).unwrap().into_raw()
}
"""

def rust_wasm_pm():
    return """use wasm_bindgen::prelude::*;
use crate::types::PackageManager;

#[wasm_bindgen]
pub fn pm_detect(path: &str) -> String {
    let pm = crate::detect(std::path::Path::new(path));
    format!("{:?}", pm)
}

#[wasm_bindgen]
pub fn pm_install_cmd(pm_str: &str) -> String {
    let pm = match pm_str {
        "Npm" => PackageManager::Npm,
        "Yarn" => PackageManager::Yarn,
        "Pnpm" => PackageManager::Pnpm,
        "Bun" => PackageManager::Bun,
        "Composer" => PackageManager::Composer,
        "Cargo" => PackageManager::Cargo,
        "Go" => PackageManager::Go,
        "Pip" => PackageManager::Pip,
        "Gem" => PackageManager::Gem,
        _ => PackageManager::Npm,
    };
    crate::install_cmd(pm).to_string()
}
"""

def rust_client_pm():
    return """use std::path::Path;
use crate::{PackageManager, detect, install_cmd, add_cmd};

pub struct PackageManagerClient {
    manager: PackageManager,
    path: std::path::PathBuf,
}

impl PackageManagerClient {
    pub fn new(manager: PackageManager, path: std::path::PathBuf) -> Self {
        Self { manager, path }
    }

    pub fn manager(&self) -> PackageManager { self.manager }

    pub fn install(&self) -> String { install_cmd(self.manager).to_string() }

    pub fn add(&self, dev: bool) -> String { add_cmd(self.manager, dev).to_string() }

    pub fn path(&self) -> &Path { &self.path }

    pub fn detect_from_path() -> PackageManager {
        detect(&std::env::current_dir().unwrap_or_default())
    }
}
"""

def rust_server_pm():
    return """use crate::{PackageManager, detect, install_cmd};
use std::sync::{Arc, Mutex};
use std::path::PathBuf;

pub struct PmServer {
    clients: Arc<Mutex<Vec<super::PackageManagerClient>>>,
}

impl PmServer {
    pub fn new() -> Self { Self { clients: Arc::new(Mutex::new(Vec::new())) } }

    pub fn add_client(&self, client: super::PackageManagerClient) {
        self.clients.lock().unwrap().push(client);
    }

    pub fn detect_and_add(&self, path: PathBuf) {
        let pm = detect(&path);
        self.add_client(super::PackageManagerClient::new(pm, path));
    }

    pub fn client_count(&self) -> usize {
        self.clients.lock().unwrap().len()
    }
}
"""

def rust_test_utils_pm():
    return """#![cfg(test)]
use crate::{PackageManager, detect, install_cmd, add_cmd};

pub fn create_temp_dir() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("klyron_pm_test_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    dir
}

#[test]
fn test_test_utils() {
    let dir = create_temp_dir();
    let pm = detect(&dir);
    assert_eq!(pm, PackageManager::Npm);
}
"""

def rust_benchmark_pm():
    return """#![cfg(test)]
use crate::PackageManager;

#[test]
fn bench_detect() {
    let start = std::time::Instant::now();
    for _ in 0..1000 {
        let _ = crate::detect(std::path::Path::new("/tmp"));
    }
    println!("detect() x1000: {:?}", start.elapsed());
}

#[test]
fn bench_install_cmd() {
    let start = std::time::Instant::now();
    for _ in 0..10000 {
        let _ = crate::install_cmd(PackageManager::Npm);
    }
    println!("install_cmd() x10000: {:?}", start.elapsed());
}
"""

def rust_lib_pm():
    return r"""pub mod types;
pub mod errors;
pub mod builder;
pub mod config;
pub mod ffi;
pub mod wasm;
pub mod client;
pub mod server;
pub mod test_utils;
pub mod benchmark;

use std::path::Path;
pub use types::PackageManager;
pub use client::PackageManagerClient;

pub fn detect(dir: &Path) -> PackageManager {
    if dir.join("yarn.lock").exists() { return PackageManager::Yarn; }
    if dir.join("pnpm-lock.yaml").exists() { return PackageManager::Pnpm; }
    if dir.join("bun.lockb").exists() { return PackageManager::Bun; }
    if dir.join("package-lock.json").exists() { return PackageManager::Npm; }
    if dir.join("composer.json").exists() { return PackageManager::Composer; }
    if dir.join("Cargo.toml").exists() { return PackageManager::Cargo; }
    if dir.join("go.mod").exists() { return PackageManager::Go; }
    if dir.join("requirements.txt").exists() || dir.join("Pipfile").exists() { return PackageManager::Pip; }
    if dir.join("Gemfile").exists() { return PackageManager::Gem; }
    PackageManager::Npm
}

pub fn install_cmd(pm: PackageManager) -> &'static str {
    match pm {
        PackageManager::Npm => "npm install",
        PackageManager::Yarn => "yarn install",
        PackageManager::Pnpm => "pnpm install",
        PackageManager::Bun => "bun install",
        PackageManager::Composer => "composer install",
        PackageManager::Cargo => "cargo build",
        PackageManager::Go => "go mod download",
        PackageManager::Pip => "pip install -r requirements.txt",
        PackageManager::Gem => "bundle install",
    }
}

pub fn add_cmd(pm: PackageManager, dev: bool) -> &'static str {
    match (pm, dev) {
        (PackageManager::Npm, false) => "npm install",
        (PackageManager::Npm, true) => "npm install --save-dev",
        (PackageManager::Yarn, false) => "yarn add",
        (PackageManager::Yarn, true) => "yarn add --dev",
        (PackageManager::Pnpm, false) => "pnpm add",
        (PackageManager::Pnpm, true) => "pnpm add --save-dev",
        (PackageManager::Bun, false) => "bun add",
        (PackageManager::Bun, true) => "bun add --dev",
        (PackageManager::Composer, false) => "composer require",
        (PackageManager::Composer, true) => "composer require --dev",
        (PackageManager::Cargo, false) => "cargo add",
        (PackageManager::Cargo, true) => "cargo add --dev",
        _ => "echo 'unsupported package manager'",
    }
}
"""

generate_rust_files("klyron_pm", rust_types_pm, rust_errors_pm, rust_builder_pm, rust_config_pm, rust_ffi_pm, rust_wasm_pm, rust_client_pm, rust_server_pm, rust_test_utils_pm, rust_benchmark_pm)

# For other crates, generate similar patterns adapted to their API...

# ============================================================
# GENERATE REMAINING 8 CRATES (simplified - using templates)
# ============================================================

def make_rust_types(crate_name, enums, structs):
    return f"""use serde::{{Deserialize, Serialize}};
{structs}
"""

def make_rust_errors(crate_name):
    return f"""use thiserror::Error;

#[derive(Error, Debug)]
pub enum {crate_name.replace('klyron_', '').upper()}Error {{
    #[error("operation failed: {{0}}")]
    OperationFailed(String),
    #[error("io error: {{0}}")]
    Io(#[from] std::io::Error),
}}
"""

def make_rust_builder(crate_name):
    return f"""use crate::config::{crate_name.replace('klyron_','').capitalize()}Config;

pub struct {crate_name.replace('klyron_','').capitalize()}Builder {{
    config: {crate_name.replace('klyron_','').capitalize()}Config,
}}

impl {crate_name.replace('klyron_','').capitalize()}Builder {{
    pub fn new() -> Self {{ Self {{ config: {crate_name.replace('klyron_','').capitalize()}Config::default() }} }}
    pub fn build(self) -> {crate_name.replace('klyron_','').capitalize()}Config {{ self.config }}
}}

impl Default for {crate_name.replace('klyron_','').capitalize()}Builder {{
    fn default() -> Self {{ Self::new() }}
}}
"""

def make_rust_config(crate_name):
    return f"""#[derive(Debug, Clone)]
pub struct {crate_name.replace('klyron_','').capitalize()}Config {{
    pub enabled: bool,
    pub verbose: bool,
}}

impl Default for {crate_name.replace('klyron_','').capitalize()}Config {{
    fn default() -> Self {{ Self {{ enabled: true, verbose: false }} }}
}}
"""

def make_rust_ffi(crate_name):
    name = crate_name.replace('klyron_', '')
    return f"""use std::ffi::CStr;
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn {name}_version() -> u32 {{
    1
}}
"""

def make_rust_wasm(crate_name):
    name = crate_name.replace('klyron_', '')
    return f"""use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn {name}_version() -> u32 {{
    1
}}
"""

def make_rust_client(crate_name):
    cap = crate_name.replace('klyron_', '').capitalize()
    return f"""pub struct {cap}Client {{
    config: crate::config::{cap}Config,
}}

impl {cap}Client {{
    pub fn new() -> Self {{ Self {{ config: crate::config::{cap}Config::default() }} }}
    pub fn config(&self) -> &crate::config::{cap}Config {{ &self.config }}
    pub fn version(&self) -> &'static str {{ "1.0.0" }}
}}
"""

def make_rust_server(crate_name):
    cap = crate_name.replace('klyron_', '').capitalize()
    return f"""use std::sync::{{Arc, Mutex}};

pub struct {cap}Server {{
    clients: Arc<Mutex<Vec<crate::client::{cap}Client>>>,
}}

impl {cap}Server {{
    pub fn new() -> Self {{ Self {{ clients: Arc::new(Mutex::new(Vec::new())) }} }}
    pub fn add_client(&self, client: crate::client::{cap}Client) {{
        self.clients.lock().unwrap().push(client);
    }}
    pub fn client_count(&self) -> usize {{ self.clients.lock().unwrap().len() }}
}}
"""

def make_rust_test_utils(crate_name):
    cap = crate_name.replace('klyron_', '').capitalize()
    return f"""#![cfg(test)]
use crate::client::{cap}Client;

pub fn create_test_client() -> {cap}Client {{
    {cap}Client::new()
}}

#[test]
fn test_create_client() {{
    let _client = create_test_client();
}}
"""

def make_rust_benchmark(crate_name):
    cap = crate_name.replace('klyron_', '').capitalize()
    return f"""#![cfg(test)]

#[test]
fn bench_version() {{
    let client = crate::client::{cap}Client::new();
    let start = std::time::Instant::now();
    for _ in 0..1000 {{ let _ = client.version(); }}
    println!("version() x1000: {{:?}}", start.elapsed());
}}
"""

def make_rust_lib(crate_name):
    cap = crate_name.replace('klyron_', '').capitalize()
    return f"""pub mod types;
pub mod errors;
pub mod builder;
pub mod config;
pub mod ffi;
pub mod wasm;
pub mod client;
pub mod server;
pub mod test_utils;
pub mod benchmark;

pub use client::{cap}Client;
pub use config::{cap}Config;
"""

def generate_all_rust(crate_name):
    if crate_name in ("klyron_napi", "klyron_pm"):
        return  # already generated
    generate_rust_files(crate_name,
        lambda: make_rust_types(crate_name, [], ""),
        lambda: make_rust_errors(crate_name),
        lambda: make_rust_builder(crate_name),
        lambda: make_rust_config(crate_name),
        lambda: make_rust_ffi(crate_name),
        lambda: make_rust_wasm(crate_name),
        lambda: make_rust_client(crate_name),
        lambda: make_rust_server(crate_name),
        lambda: make_rust_test_utils(crate_name),
        lambda: make_rust_benchmark(crate_name),
    )

# Typescript generators for generic crates
def make_ts_types(crate_name):
    cap = crate_name.replace('klyron_', '').capitalize()
    return f"""export interface {cap}Config {{
  enabled: boolean;
  verbose: boolean;
}}

export interface {cap}Result {{
  success: boolean;
  message: string;
}}
"""

def make_ts_client(crate_name):
    cap = crate_name.replace('klyron_', '').capitalize()
    return f"""import {{ {cap}Config }} from './types';

export class {cap}Client {{
  private config: {cap}Config;

  constructor(config?: Partial<{cap}Config>) {{
    this.config = {{ enabled: true, verbose: false, ...config }};
  }}

  version(): string {{ return '1.0.0'; }}

  getConfig(): {cap}Config {{ return this.config; }}
}}
"""

def make_ts_errors(crate_name):
    cap = crate_name.replace('klyron_', '').capitalize()
    return f"""export class {cap}Error extends Error {{
  constructor(message: string) {{
    super(message);
    this.name = '{cap}Error';
  }}
}}
"""

def make_ts_config(crate_name):
    cap = crate_name.replace('klyron_', '').capitalize()
    return f"""import {{ {cap}Config }} from './types';

export const DEFAULT_{cap.upper()}_CONFIG: {cap}Config = {{
  enabled: true,
  verbose: false,
}};

export function create{cap}Config(overrides?: Partial<{cap}Config>): {cap}Config {{
  return {{ ...DEFAULT_{cap.upper()}_CONFIG, ...overrides }};
}}
"""

def make_ts_utils(crate_name):
    cap = crate_name.replace('klyron_', '').capitalize()
    return f"""export function format{cap}Message(msg: string): string {{
  return `[{cap}] ${{msg}}`;
}}
"""

def make_ts_index(crate_name):
    cap = crate_name.replace('klyron_', '').capitalize()
    return f"""export {{ {cap}Client }} from './client';
export {{ {cap}Error }} from './errors';
export {{ create{cap}Config, DEFAULT_{cap.upper()}_CONFIG }} from './config';
export {{ {cap}Config, {cap}Result }} from './types';
export {{ format{cap}Message }} from './utils';
"""

def make_ts_test(crate_name):
    cap = crate_name.replace('klyron_', '').capitalize()
    return f"""import {{ {cap}Client }} from './client';

export function createTest{cap}Client(): {cap}Client {{
  return new {cap}Client();
}}
"""

def make_ts_benchmark(crate_name):
    cap = crate_name.replace('klyron_', '').capitalize()
    return f"""import {{ {cap}Client }} from './client';

export function bench{cap}Version(iterations: number): number {{
  const client = new {cap}Client();
  const start = process.hrtime.bigint();
  for (let i = 0; i < iterations; i++) {{ client.version(); }}
  return Number(process.hrtime.bigint() - start) / 1e6;
}}
"""

def make_ts_fixtures(crate_name):
    cap = crate_name.replace('klyron_', '').capitalize()
    return f"""import {{ {cap}Config }} from './types';

export const TEST_{cap.upper()}_CONFIGS: {cap}Config[] = [
  {{ enabled: true, verbose: false }},
  {{ enabled: false, verbose: true }},
];
"""

def make_ts_declaration(crate_name):
    cap = crate_name.replace('klyron_', '').capitalize()
    return f"""declare module '@klyron/{crate_name.replace("klyron_", "")}' {{
  export class {cap}Client {{
    constructor(config?: Partial<{{ enabled: boolean; verbose: boolean }}>);
    version(): string;
  }}
}}
"""

def generate_all_ts(crate_name):
    if crate_name == "klyron_napi":
        return
    generate_ts_files(crate_name,
        lambda: make_ts_types(crate_name),
        lambda: make_ts_client(crate_name),
        lambda: make_ts_errors(crate_name),
        lambda: make_ts_config(crate_name),
        lambda: make_ts_utils(crate_name),
        lambda: make_ts_index(crate_name),
        lambda: make_ts_test(crate_name),
        lambda: make_ts_benchmark(crate_name),
        lambda: make_ts_fixtures(crate_name),
        lambda: make_ts_declaration(crate_name),
    )

# C++ generators for generic crates
def make_cpp_types_hpp(crate_name):
    cap = crate_name.replace('klyron_', '').capitalize()
    return f"""#pragma once
#include <string>
#include <vector>

namespace klyron_{crate_name.replace('klyron_', '')} {{

struct {cap}Config {{
    bool enabled = true;
    bool verbose = false;
}};

struct {cap}Result {{
    bool success = false;
    std::string message;
}};

}} // namespace
"""

def make_cpp_api_hpp(crate_name):
    cap = crate_name.replace('klyron_', '').capitalize()
    return f"""#pragma once
#include "types.hpp"
#include <memory>

namespace klyron_{crate_name.replace('klyron_', '')} {{

class {cap}Client {{
public:
    {cap}Client();
    explicit {cap}Client(const {cap}Config& config);
    std::string version() const;
    {cap}Config config() const;

private:
    {cap}Config config_;
}};

}} // namespace
"""

def make_cpp_config_hpp(crate_name):
    cap = crate_name.replace('klyron_', '').capitalize()
    return f"""#pragma once
#include "types.hpp"

namespace klyron_{crate_name.replace('klyron_', '')} {{

class {cap}ConfigManager {{
public:
    static {cap}Config defaults();
}};

}} // namespace
"""

def make_cpp_errors_hpp(crate_name):
    cap = crate_name.replace('klyron_', '').capitalize()
    return f"""#pragma once
#include <stdexcept>
#include <string>

namespace klyron_{crate_name.replace('klyron_', '')} {{

class {cap}Exception : public std::runtime_error {{
public:
    explicit {cap}Exception(const std::string& msg) : std::runtime_error(msg) {{}}
}};

}} // namespace
"""

def make_cpp_builder_hpp(crate_name):
    cap = crate_name.replace('klyron_', '').capitalize()
    return f"""#pragma once
#include "types.hpp"

namespace klyron_{crate_name.replace('klyron_', '')} {{

class {cap}Builder {{
public:
    {cap}Builder();
    {cap}Builder& enabled(bool v);
    {cap}Builder& verbose(bool v);
    {cap}Config build();
private:
    {cap}Config config_;
}};

}} // namespace
"""

def make_cpp_ffi_hpp(crate_name):
    name = crate_name.replace('klyron_', '')
    return f"""#pragma once
#include <cstdint>

extern "C" {{
uint32_t {name}_version(void);
}}
"""

# Generic C++ .cpp files
def make_cpp_types_cpp():
    return """#include "types.hpp"
// Types are header-only
"""

def make_cpp_api_cpp(crate_name):
    cap = crate_name.replace('klyron_', '').capitalize()
    return f"""#include "api.hpp"

namespace klyron_{crate_name.replace('klyron_', '')} {{

{cap}Client::{cap}Client() : config_() {{}}
{cap}Client::{cap}Client(const {cap}Config& config) : config_(config) {{}}
std::string {cap}Client::version() const {{ return "1.0.0"; }}
{cap}Config {cap}Client::config() const {{ return config_; }}

}} // namespace
"""

def make_cpp_config_cpp(crate_name):
    cap = crate_name.replace('klyron_', '').capitalize()
    return f"""#include "config.hpp"

namespace klyron_{crate_name.replace('klyron_', '')} {{
{cap}Config {cap}ConfigManager::defaults() {{ return {cap}Config(); }}
}}
"""

def make_cpp_errors_cpp():
    return """#include "errors.hpp"
// Error classes are header-only
"""

def make_cpp_builder_cpp(crate_name):
    cap = crate_name.replace('klyron_', '').capitalize()
    return f"""#include "builder.hpp"

namespace klyron_{crate_name.replace('klyron_', '')} {{
{cap}Builder::{cap}Builder() : config_() {{}}
{cap}Builder& {cap}Builder::enabled(bool v) {{ config_.enabled = v; return *this; }}
{cap}Builder& {cap}Builder::verbose(bool v) {{ config_.verbose = v; return *this; }}
{cap}Config {cap}Builder::build() {{ return config_; }}
}}
"""

def make_cpp_ffi_cpp(crate_name):
    name = crate_name.replace('klyron_', '')
    return f"""#include "ffi.hpp"

extern "C" {{
uint32_t {name}_version(void) {{ return 1; }}
}}
"""

def generate_all_cpp(crate_name):
    if crate_name == "klyron_napi":
        return
    generate_cpp_files(crate_name,
        lambda: make_cpp_types_hpp(crate_name),
        make_cpp_types_cpp,
        lambda: make_cpp_api_hpp(crate_name),
        lambda: make_cpp_api_cpp(crate_name),
        lambda: make_cpp_config_hpp(crate_name),
        lambda: make_cpp_config_cpp(crate_name),
        lambda: make_cpp_errors_hpp(crate_name),
        make_cpp_errors_cpp,
        lambda: make_cpp_builder_hpp(crate_name),
        lambda: make_cpp_builder_cpp(crate_name),
        lambda: make_cpp_ffi_hpp(crate_name),
        lambda: make_cpp_ffi_cpp(crate_name),
    )

# C generators for generic crates
def make_c_header_h(crate_name):
    name = crate_name.replace('klyron_', '')
    cap = name.capitalize()
    return f"""#ifndef KLYRON_{name.upper()}_H
#define KLYRON_{name.upper()}_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {{
#endif

typedef struct {{
    bool enabled;
    bool verbose;
}} klyron_{name}_config_t;

uint32_t klyron_{name}_version(void);
klyron_{name}_config_t klyron_{name}_config_default(void);

#ifdef __cplusplus
}}
#endif

#endif /* KLYRON_{name.upper()}_H */
"""

def make_c_impl_c(crate_name):
    name = crate_name.replace('klyron_', '')
    return f"""#include "klyron_{name}.h"

uint32_t klyron_{name}_version(void) {{ return 1; }}

klyron_{name}_config_t klyron_{name}_config_default(void) {{
    klyron_{name}_config_t config = {{ .enabled = true, .verbose = false }};
    return config;
}}
"""

def make_c_types_h(crate_name):
    name = crate_name.replace('klyron_', '')
    return f"""#ifndef KLYRON_{name.upper()}_TYPES_H
#define KLYRON_{name.upper()}_TYPES_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {{
#endif

typedef struct {{
    bool enabled;
    bool verbose;
}} klyron_{name}_config_t;

#ifdef __cplusplus
}}
#endif

#endif /* KLYRON_{name.upper()}_TYPES_H */
"""

def make_c_errors_h(crate_name):
    name = crate_name.replace('klyron_', '')
    return f"""#ifndef KLYRON_{name.upper()}_ERRORS_H
#define KLYRON_{name.upper()}_ERRORS_H

#ifdef __cplusplus
extern "C" {{
#endif

typedef enum {{
    KLYRON_{name.upper()}_OK = 0,
    KLYRON_{name.upper()}_ERR_FAILED = -1,
}} klyron_{name}_error_t;

const char* klyron_{name}_error_string(klyron_{name}_error_t err);

#ifdef __cplusplus
}}
#endif

#endif /* KLYRON_{name.upper()}_ERRORS_H */
"""

def make_c_errors_c(crate_name):
    name = crate_name.replace('klyron_', '')
    return f"""#include "errors.h"

const char* klyron_{name}_error_string(klyron_{name}_error_t err) {{
    switch (err) {{
        case KLYRON_{name.upper()}_OK: return "success";
        case KLYRON_{name.upper()}_ERR_FAILED: return "operation failed";
        default: return "unknown error";
    }}
}}
"""

def make_c_config_h(crate_name):
    name = crate_name.replace('klyron_', '')
    return f"""#ifndef KLYRON_{name.upper()}_CONFIG_H
#define KLYRON_{name.upper()}_CONFIG_H

#include "types.h"

#ifdef __cplusplus
extern "C" {{
#endif

klyron_{name}_config_t klyron_{name}_config_default(void);

#ifdef __cplusplus
}}
#endif

#endif /* KLYRON_{name.upper()}_CONFIG_H */
"""

def make_c_config_c(crate_name):
    name = crate_name.replace('klyron_', '')
    return f"""#include "config.h"
#include <stdlib.h>

klyron_{name}_config_t klyron_{name}_config_default(void) {{
    klyron_{name}_config_t config = {{ .enabled = true, .verbose = false }};
    return config;
}}
"""

def make_c_utils_h(crate_name):
    name = crate_name.replace('klyron_', '')
    return f"""#ifndef KLYRON_{name.upper()}_UTILS_H
#define KLYRON_{name.upper()}_UTILS_H

#ifdef __cplusplus
extern "C" {{
#endif

void klyron_{name}_version_str(char* buf, size_t len);

#ifdef __cplusplus
}}
#endif

#endif /* KLYRON_{name.upper()}_UTILS_H */
"""

def make_c_utils_c(crate_name):
    name = crate_name.replace('klyron_', '')
    return f"""#include "utils.h"
#include <string.h>

void klyron_{name}_version_str(char* buf, size_t len) {{
    if (buf && len > 0) {{
        snprintf(buf, len, "%d.%d.%d", 1, 0, 0);
    }}
}}
"""

def generate_all_c(crate_name):
    if crate_name == "klyron_napi":
        return
    generate_c_files(crate_name,
        lambda: make_c_header_h(crate_name),
        lambda: make_c_impl_c(crate_name),
        lambda: make_c_types_h(crate_name),
        lambda: make_c_errors_h(crate_name),
        lambda: make_c_errors_c(crate_name),
        lambda: make_c_config_h(crate_name),
        lambda: make_c_config_c(crate_name),
        lambda: make_c_utils_h(crate_name),
        lambda: make_c_utils_c(crate_name),
    )

# Generate for all remaining crates
for crate in CRATES:
    if crate == "klyron_napi":
        continue
    print(f"Generating bindings for {crate}...")
    generate_all_rust(crate)
    generate_all_ts(crate)
    generate_all_cpp(crate)
    generate_all_c(crate)
    generate_bash_files(crate)

print("All binding files generated successfully!")
print(f"Total crates processed: {len(CRATES)}")
