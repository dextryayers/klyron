#!/usr/bin/env python3
"""Generate all binding files for 9 crates and 4 engines."""

import os, shutil

ROOT = "/home/aniipid/koding/klyronjs"

CRATES = ["klyron_updater","klyron_utils","klyron_cli","klyron_runtime","klyron_engine","klyron_sqlite","klyron_postgres","klyron_mysql","klyron_ai"]
ENGINES = ["v8","boa","quickjs","jsc"]

CRATE_DESC = {
    "klyron_updater": "Self-update mechanism",
    "klyron_utils": "Shared utilities",
    "klyron_cli": "CLI entry point",
    "klyron_runtime": "Core runtime",
    "klyron_engine": "Polyglot engine trait + bridges",
    "klyron_sqlite": "SQLite binding",
    "klyron_postgres": "PostgreSQL binding",
    "klyron_mysql": "MySQL binding",
    "klyron_ai": "AI features",
}

def cap(name):
    return name.replace("klyron_","klyron::").replace("_"," ").title().replace(" ","")

def crate_upper(name):
    return name.upper()

def crate_rust_use(name):
    return name.replace("-","_")

# ---------------------------------------------------------------------------
# 1. Rust bindings (10 files)
# ---------------------------------------------------------------------------
def gen_rust_types(crate):
    c = crate.replace("klyron_","")
    return f'''//! Type definitions for {crate}

use serde::{{Deserialize, Serialize}};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {cap(crate)}Config {{
    pub enabled: bool,
}}

impl Default for {cap(crate)}Config {{
    fn default() -> Self {{
        Self {{ enabled: true }}
    }}
}}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {cap(crate)}Result<T> {{
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}}

impl<T> {cap(crate)}Result<T> {{
    pub fn ok(data: T) -> Self {{
        Self {{ success: true, data: Some(data), error: None }}
    }}

    pub fn err(msg: impl Into<String>) -> Self {{
        Self {{ success: false, data: None, error: Some(msg.into()) }}
    }}
}}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum {cap(crate)}Status {{
    Active,
    Inactive,
    Error(String),
}}
'''

def gen_rust_errors(crate):
    return f'''//! Error types for {crate}

use std::fmt;

#[derive(Debug)]
pub enum {cap(crate)}Error {{
    NotFound(String),
    InvalidInput(String),
    OperationFailed(String),
    IoError(std::io::Error),
}}

impl fmt::Display for {cap(crate)}Error {{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {{
        match self {{
            Self::NotFound(msg) => write!(f, "NotFound: {{msg}}"),
            Self::InvalidInput(msg) => write!(f, "InvalidInput: {{msg}}"),
            Self::OperationFailed(msg) => write!(f, "OperationFailed: {{msg}}"),
            Self::IoError(e) => write!(f, "IoError: {{e}}"),
        }}
    }}
}}

impl std::error::Error for {cap(crate)}Error {{}}

impl From<std::io::Error> for {cap(crate)}Error {{
    fn from(e: std::io::Error) -> Self {{
        Self::IoError(e)
    }}
}}

impl From<{cap(crate)}Error> for anyhow::Error {{
    fn from(e: {cap(crate)}Error) -> Self {{
        anyhow::anyhow!("{{}}", e)
    }}
}}
'''

def gen_rust_builder(crate):
    return f'''//! Builder pattern for {crate}

use crate::types::{cap(crate)}Config;

#[derive(Debug, Default)]
pub struct {cap(crate)}Builder {{
    config: Option<{cap(crate)}Config>,
    verbose: bool,
}}

impl {cap(crate)}Builder {{
    pub fn new() -> Self {{
        Self::default()
    }}

    pub fn with_config(mut self, config: {cap(crate)}Config) -> Self {{
        self.config = Some(config);
        self
    }}

    pub fn verbose(mut self, verbose: bool) -> Self {{
        self.verbose = verbose;
        self
    }}

    pub fn build(self) -> anyhow::Result<{cap(crate)}Instance> {{
        let config = self.config.unwrap_or_default();
        Ok({cap(crate)}Instance {{ config, verbose: self.verbose }})
    }}
}}

#[derive(Debug)]
pub struct {cap(crate)}Instance {{
    pub config: {cap(crate)}Config,
    pub verbose: bool,
}}
'''

def gen_rust_config(crate):
    return f'''//! Configuration for {crate}

use serde::{{Deserialize, Serialize}};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {cap(crate)}Settings {{
    pub max_retries: u32,
    pub timeout_ms: u64,
    pub log_level: String,
}}

impl Default for {cap(crate)}Settings {{
    fn default() -> Self {{
        Self {{ max_retries: 3, timeout_ms: 5000, log_level: "info".into() }}
    }}
}}

pub fn load_config(path: Option<&std::path::Path>) -> anyhow::Result<{cap(crate)}Settings> {{
    if let Some(p) = path {{
        let content = std::fs::read_to_string(p)?;
        Ok(serde_json::from_str(&content)?)
    }} else {{
        Ok({cap(crate)}Settings::default())
    }}
}}
'''

def gen_rust_ffi(crate):
    ffi_name = crate_upper(crate)
    return f'''//! FFI bindings for {crate}

use std::ffi::CStr;
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn {crate}_init() -> i32 {{
    0
}}

#[no_mangle]
pub extern "C" fn {crate}_version() -> *const c_char {{
    concat!("{crate} v", env!("CARGO_PKG_VERSION"), "\\0").as_ptr() as *const c_char
}}

#[no_mangle]
pub extern "C" fn {crate}_process(input: *const c_char) -> *mut c_char {{
    if input.is_null() {{
        return std::ffi::CString::new("error: null input").unwrap().into_raw();
    }}
    let s = unsafe {{ CStr::from_ptr(input) }};
    let _msg = s.to_string_lossy();
    std::ffi::CString::new("ok").unwrap().into_raw()
}}

#[no_mangle]
pub extern "C" fn {crate}_free_string(s: *mut c_char) {{
    if !s.is_null() {{ unsafe {{ let _ = std::ffi::CString::from_raw(s); }} }}
}}
'''

def gen_rust_wasm(crate):
    return f'''//! WASM bindings for {crate}

pub fn wasm_init() -> anyhow::Result<()> {{
    Ok(())
}}

pub fn wasm_process(input: &str) -> String {{
    format!("{{input}} processed by {crate}")
}}
'''

def gen_rust_client(crate):
    return f'''//! Client for {crate}

pub struct {cap(crate)}Client {{
    endpoint: String,
}}

impl {cap(crate)}Client {{
    pub fn new(endpoint: impl Into<String>) -> Self {{
        Self {{ endpoint: endpoint.into() }}
    }}

    pub fn endpoint(&self) -> &str {{
        &self.endpoint
    }}

    pub fn connect(&self) -> anyhow::Result<()> {{
        Ok(())
    }}

    pub fn ping(&self) -> anyhow::Result<bool> {{
        Ok(true)
    }}
}}
'''

def gen_rust_server(crate):
    return f'''//! Server for {crate}

pub struct {cap(crate)}Server {{
    addr: String,
}}

impl {cap(crate)}Server {{
    pub fn new(addr: impl Into<String>) -> Self {{
        Self {{ addr: addr.into() }}
    }}

    pub fn start(&self) -> anyhow::Result<()> {{
        println!("{{}} server starting on {{}}", "{crate}", self.addr);
        Ok(())
    }}

    pub fn stop(&self) -> anyhow::Result<()> {{
        Ok(())
    }}
}}
'''

def gen_rust_test_utils(crate):
    return f'''//! Test utilities for {crate}

pub fn setup_test_env() {{
    let _ = tracing_subscriber::fmt().with_test_writer().try_init();
}}

pub fn create_test_config() -> crate::types::{cap(crate)}Config {{
    crate::types::{cap(crate)}Config::default()
}}
'''

def gen_rust_lib(crate):
    return f'''//! {crate} — {CRATE_DESC[crate]}
//!
//! ## Crate bindings
//! This module exposes {crate} across Rust, TypeScript, C++, C, and Bash.

pub mod types;
pub mod errors;
pub mod builder;
pub mod config;
pub mod ffi;
pub mod wasm;
pub mod client;
pub mod server;
pub mod test_utils;

pub use types::{{{cap(crate)}Config, {cap(crate)}Result, {cap(crate)}Status}};
pub use errors::{{{cap(crate)}Error}};
pub use builder::{{{cap(crate)}Builder, {cap(crate)}Instance}};
pub use config::{{{cap(crate)}Settings}};
pub use client::{{{cap(crate)}Client}};
pub use server::{{{cap(crate)}Server}};
'''

# ---------------------------------------------------------------------------
# 2. TypeScript bindings (10 files)
# ---------------------------------------------------------------------------
def gen_ts_types(crate):
    return f'''// Type definitions for {crate}
export interface {cap(crate).replace(" ","")}Config {{
  enabled: boolean;
}}

export interface {cap(crate).replace(" ","")}Result<T> {{
  success: boolean;
  data: T | null;
  error: string | null;
}}

export enum {cap(crate).replace(" ","")}Status {{
  Active = "active",
  Inactive = "inactive",
  Error = "error",
}}

export type {cap(crate).replace(" ","")}Options = {{
  verbose?: boolean;
  timeout?: number;
}};
'''

def gen_ts_client(crate):
    cap_name = cap(crate).replace(" ","")
    return f'''// Client for {crate}
import {{ {cap_name}Config, {cap_name}Result }} from "./types";

export class {cap_name}Client {{
  private endpoint: string;

  constructor(endpoint: string) {{
    this.endpoint = endpoint;
  }}

  async connect(): Promise<{cap_name}Result<null>> {{
    return {{ success: true, data: null, error: null }};
  }}

  async ping(): Promise<boolean> {{
    return true;
  }}
}}
'''

def gen_ts_errors(crate):
    cap_name = cap(crate).replace(" ","")
    return f'''// Error types for {crate}
export class {cap_name}Error extends Error {{
  constructor(message: string) {{
    super(message);
    this.name = "{cap_name}Error";
  }}
}}

export class NotFoundError extends {cap_name}Error {{
  constructor(resource: string) {{
    super(`Not found: ${{resource}}`);
    this.name = "NotFoundError";
  }}
}}

export class InvalidInputError extends {cap_name}Error {{
  constructor(detail: string) {{
    super(`Invalid input: ${{detail}}`);
    this.name = "InvalidInputError";
  }}
}}
'''

def gen_ts_config(crate):
    cap_name = cap(crate).replace(" ","")
    return f'''// Configuration for {crate}
import {{ {cap_name}Config }} from "./types";

const DEFAULT_CONFIG: {cap_name}Config = {{
  enabled: true,
}};

export function loadConfig(path?: string): {cap_name}Config {{
  if (path) {{
    try {{
      return JSON.parse(Deno.readTextFileSync(path));
    }} catch {{
      return DEFAULT_CONFIG;
    }}
  }}
  return DEFAULT_CONFIG;
}}

export interface {cap_name}Settings {{
  maxRetries: number;
  timeoutMs: number;
  logLevel: string;
}}

export const defaultSettings: {cap_name}Settings = {{
  maxRetries: 3,
  timeoutMs: 5000,
  logLevel: "info",
}};
'''

def gen_ts_utils(crate):
    cap_name = cap(crate).replace(" ","")
    return f'''// Utilities for {crate}
import {{ {cap_name}Config }} from "./types.ts";

export function formatConfig(config: {cap_name}Config): string {{
  return JSON.stringify(config, null, 2);
}}

export function delay(ms: number): Promise<void> {{
  return new Promise(resolve => setTimeout(resolve, ms));
}}

export function createTempDir(): string {{
  const dir = `/tmp/{crate}_${{Date.now()}}`;
  Deno.mkdirSync(dir, {{ recursive: true }});
  return dir;
}}
'''

def gen_ts_index(crate):
    cap_name = cap(crate).replace(" ","")
    return f'''// {crate} — {CRATE_DESC[crate]}
export * from "./types.ts";
export * from "./client.ts";
export * from "./errors.ts";
export * from "./config.ts";
export * from "./utils.ts";

import {{ {cap_name}Client }} from "./client.ts";
import {{ loadConfig }} from "./config.ts";

export function createClient(endpoint: string, configPath?: string): {cap_name}Client {{
  const _config = loadConfig(configPath);
  return new {cap_name}Client(endpoint);
}}
'''

def gen_ts_test(crate):
    cap_name = cap(crate).replace(" ","")
    return f'''// Tests for {crate}
import {{ assertEquals }} from "https://deno.land/std/assert/mod.ts";
import {{ {cap_name}Client }} from "./client.ts";
import {{ loadConfig }} from "./config.ts";

Deno.test("{crate} client connect", async () => {{
  const client = new {cap_name}Client("http://localhost:8080");
  const result = await client.connect();
  assertEquals(result.success, true);
}});

Deno.test("{crate} load default config", () => {{
  const config = loadConfig();
  assertEquals(config.enabled, true);
}});
'''

def gen_ts_benchmark(crate):
    cap_name = cap(crate).replace(" ","")
    return f'''// Benchmarks for {crate}
import {{ {cap_name}Client }} from "./client.ts";

Deno.bench("{crate} ping", async () => {{
  const client = new {cap_name}Client("http://localhost:8080");
  await client.ping();
}});
'''

def gen_ts_fixtures(crate):
    cap_name = cap(crate).replace(" ","")
    return f'''// Test fixtures for {crate}
import {{ {cap_name}Config }} from "./types.ts";

export const sampleConfig: {cap_name}Config = {{
  enabled: true,
}};

export const invalidConfig: Record<string, unknown> = {{
  enabled: "not-a-boolean",
}};
'''

def gen_ts_declaration(crate):
    cap_name = cap(crate).replace(" ","")
    return f'''// Type declarations for {crate}
declare module "klyron/{crate}" {{
  export interface {cap_name}Config {{
    enabled: boolean;
  }}

  export interface {cap_name}Result<T> {{
    success: boolean;
    data: T | null;
    error: string | null;
  }}

  export class {cap_name}Client {{
    constructor(endpoint: string);
    connect(): Promise<{cap_name}Result<null>>;
    ping(): Promise<boolean>;
  }}

  export function createClient(endpoint: string, configPath?: string): {cap_name}Client;
  export function loadConfig(path?: string): {cap_name}Config;
}}
'''

# ---------------------------------------------------------------------------
# 3. C++ bindings (11 files)
# ---------------------------------------------------------------------------
def gen_cpp_types_hpp(crate):
    cap_name = cap(crate).replace(" ","")
    guard = f"{crate.upper()}_BINDINGS_TYPES_HPP"
    return f'''#ifndef {guard}
#define {guard}

#include <string>
#include <vector>
#include <memory>
#include <functional>

namespace klyron {{

struct {cap_name}Config {{
  bool enabled = true;
}};

struct {cap_name}Result {{
  bool success = false;
  std::string data;
  std::string error;
}};

}} // namespace klyron

#endif
'''

def gen_cpp_types_cpp(crate):
    cap_name = cap(crate).replace(" ","")
    return f'''#include "types.hpp"

namespace klyron {{
// {cap_name} type implementations
}}
'''

def gen_cpp_api_hpp(crate):
    cap_name = cap(crate).replace(" ","")
    guard = f"{crate.upper()}_BINDINGS_API_HPP"
    return f'''#ifndef {guard}
#define {guard}

#include "types.hpp"
#include "config.hpp"

namespace klyron {{

class {cap_name}Api {{
public:
  {cap_name}Api();
  ~{cap_name}Api();

  {cap_name}Result process(const std::string& input);
  std::string version() const;
  bool ping();

private:
  class Impl;
  std::unique_ptr<Impl> impl_;
}};

}} // namespace klyron

#endif
'''

def gen_cpp_api_cpp(crate):
    cap_name = cap(crate).replace(" ","")
    return f'''#include "api.hpp"

namespace klyron {{

class {cap_name}Api::Impl {{
public:
  bool initialized = false;
}};

{cap_name}Api::{cap_name}Api()
  : impl_(std::make_unique<Impl>()) {{
}}

{cap_name}Api::~{cap_name}Api() = default;

{cap_name}Result {cap_name}Api::process(const std::string& input) {{
  {cap_name}Result result;
  result.success = true;
  result.data = "processed: " + input;
  return result;
}}

std::string {cap_name}Api::version() const {{
  return "{crate} 0.1.0";
}}

bool {cap_name}Api::ping() {{
  return true;
}}

}} // namespace klyron
'''

def gen_cpp_config_hpp(crate):
    cap_name = cap(crate).replace(" ","")
    guard = f"{crate.upper()}_BINDINGS_CONFIG_HPP"
    return f'''#ifndef {guard}
#define {guard}

#include "types.hpp"
#include <string>

namespace klyron {{

struct {cap_name}Settings {{
  int maxRetries = 3;
  long timeoutMs = 5000;
  std::string logLevel = "info";
}};

{cap_name}Config loadConfig(const std::string& path = "");

}} // namespace klyron

#endif
'''

def gen_cpp_config_cpp(crate):
    cap_name = cap(crate).replace(" ","")
    return f'''#include "config.hpp"
#include <fstream>
#include <nlohmann/json.hpp>

namespace klyron {{

{cap_name}Config loadConfig(const std::string& path) {{
  {cap_name}Config config;
  if (!path.empty()) {{
    std::ifstream file(path);
    if (file.is_open()) {{
      try {{
        auto json = nlohmann::json::parse(file);
        config.enabled = json.value("enabled", true);
      }} catch (...) {{}}
    }}
  }}
  return config;
}}

}} // namespace klyron
'''

def gen_cpp_errors_hpp(crate):
    cap_name = cap(crate).replace(" ","")
    guard = f"{crate.upper()}_BINDINGS_ERRORS_HPP"
    return f'''#ifndef {guard}
#define {guard}

#include <stdexcept>
#include <string>

namespace klyron {{

class {cap_name}Exception : public std::runtime_error {{
public:
  explicit {cap_name}Exception(const std::string& msg)
    : std::runtime_error(msg) {{}}
}};

class NotFoundException : public {cap_name}Exception {{
public:
  explicit NotFoundException(const std::string& resource)
    : {cap_name}Exception("Not found: " + resource) {{}}
}};

class InvalidInputException : public {cap_name}Exception {{
public:
  explicit InvalidInputException(const std::string& detail)
    : {cap_name}Exception("Invalid input: " + detail) {{}}
}};

}} // namespace klyron

#endif
'''

def gen_cpp_errors_cpp(crate):
    cap_name = cap(crate).replace(" ","")
    return f'''#include "errors.hpp"

namespace klyron {{
// {cap_name} exception implementations
}}
'''

def gen_cpp_builder_hpp(crate):
    cap_name = cap(crate).replace(" ","")
    guard = f"{crate.upper()}_BINDINGS_BUILDER_HPP"
    return f'''#ifndef {guard}
#define {guard}

#include "types.hpp"
#include "config.hpp"
#include <memory>

namespace klyron {{

class {cap_name}Builder {{
public:
  {cap_name}Builder();
  {cap_name}Builder& withConfig(const {cap_name}Config& config);
  {cap_name}Builder& verbose(bool v);
  class {cap_name}Instance build();

private:
  std::unique_ptr<class Impl> impl_;
}};

class {cap_name}Builder::{cap_name}Instance {{
public:
  {cap_name}Config config;
  bool verbose = false;
}};

}} // namespace klyron

#endif
'''

def gen_cpp_builder_cpp(crate):
    cap_name = cap(crate).replace(" ","")
    return f'''#include "builder.hpp"

namespace klyron {{

class {cap_name}Builder::Impl {{
public:
  {cap_name}Config config;
  bool verbose = false;
}};

{cap_name}Builder::{cap_name}Builder()
  : impl_(std::make_unique<Impl>()) {{
}}

{cap_name}Builder& {cap_name}Builder::withConfig(const {cap_name}Config& config) {{
  impl_->config = config;
  return *this;
}}

{cap_name}Builder& {cap_name}Builder::verbose(bool v) {{
  impl_->verbose = v;
  return *this;
}}

{cap_name}Builder::{cap_name}Instance {cap_name}Builder::build() {{
  {cap_name}Instance inst;
  inst.config = impl_->config;
  inst.verbose = impl_->verbose;
  return inst;
}}

}} // namespace klyron
'''

def gen_cpp_ffi_hpp(crate):
    cap_name = cap(crate).replace(" ","")
    guard = f"{crate.upper()}_BINDINGS_FFI_HPP"
    return f'''#ifndef {guard}
#define {guard}

#include "types.hpp"
#include <functional>

namespace klyron {{

extern "C" {{
  int {crate}_init();
  const char* {crate}_version();
  char* {crate}_process(const char* input);
  void {crate}_free_string(char* s);
}}

}} // namespace klyron

#endif
'''

# ---------------------------------------------------------------------------
# 4. C bindings (10 files)
# ---------------------------------------------------------------------------
def gen_c_types_h(crate):
    guard = f"{crate.upper()}_BINDINGS_TYPES_H"
    return f'''#ifndef {guard}
#define {guard}

#include <stdbool.h>
#include <stdint.h>

typedef struct {crate}_config_t {{
  bool enabled;
}} {crate}_config_t;

typedef struct {crate}_result_t {{
  bool success;
  char* data;
  char* error;
}} {crate}_result_t;

typedef enum {crate}_status_t {{
  STATUS_ACTIVE,
  STATUS_INACTIVE,
  STATUS_ERROR
}} {crate}_status_t;

#endif
'''

def gen_c_types_c(crate):
    return f'''#include "types.h"
// {crate} type implementations
'''

def gen_c_api_h(crate):
    guard = f"{crate.upper()}_BINDINGS_API_H"
    return f'''#ifndef {guard}
#define {guard}

#include "types.h"
#include "config.h"

{crate}_result_t* {crate}_process(const char* input);
const char* {crate}_version(void);
bool {crate}_ping(void);
void {crate}_result_free({crate}_result_t* result);

#endif
'''

def gen_c_api_c(crate):
    return f'''#include "api.h"
#include <stdlib.h>
#include <string.h>

{crate}_result_t* {crate}_process(const char* input) {{
  {crate}_result_t* result = ({crate}_result_t*)malloc(sizeof({crate}_result_t));
  if (!result) return NULL;
  result->success = true;
  result->data = strdup("processed");
  result->error = NULL;
  return result;
}}

const char* {crate}_version(void) {{
  return "{crate} 0.1.0";
}}

bool {crate}_ping(void) {{
  return true;
}}

void {crate}_result_free({crate}_result_t* result) {{
  if (result) {{
    free(result->data);
    free(result->error);
    free(result);
  }}
}}
'''

def gen_c_config_h(crate):
    guard = f"{crate.upper()}_BINDINGS_CONFIG_H"
    return f'''#ifndef {guard}
#define {guard}

#include "types.h"

typedef struct {crate}_settings_t {{
  int max_retries;
  long timeout_ms;
  char* log_level;
}} {crate}_settings_t;

void {crate}_config_init({crate}_config_t* config);
{crate}_settings_t {crate}_settings_default(void);

#endif
'''

def gen_c_config_c(crate):
    return f'''#include "config.h"
#include <stdlib.h>
#include <string.h>

void {crate}_config_init({crate}_config_t* config) {{
  if (config) {{
    config->enabled = true;
  }}
}}

{crate}_settings_t {crate}_settings_default(void) {{
  {crate}_settings_t s;
  s.max_retries = 3;
  s.timeout_ms = 5000;
  s.log_level = strdup("info");
  return s;
}}
'''

def gen_c_errors_h(crate):
    guard = f"{crate.upper()}_BINDINGS_ERRORS_H"
    return f'''#ifndef {guard}
#define {guard}

typedef enum {crate}_error_code_t {{
  ERROR_NONE = 0,
  ERROR_NOT_FOUND,
  ERROR_INVALID_INPUT,
  ERROR_OPERATION_FAILED
}} {crate}_error_code_t;

const char* {crate}_error_message({crate}_error_code_t code);

#endif
'''

def gen_c_errors_c(crate):
    return f'''#include "errors.h"

const char* {crate}_error_message({crate}_error_code_t code) {{
  switch (code) {{
    case ERROR_NONE: return "ok";
    case ERROR_NOT_FOUND: return "not found";
    case ERROR_INVALID_INPUT: return "invalid input";
    case ERROR_OPERATION_FAILED: return "operation failed";
    default: return "unknown error";
  }}
}}
'''

def gen_c_ffi_h(crate):
    guard = f"{crate.upper()}_BINDINGS_FFI_H"
    return f'''#ifndef {guard}
#define {guard}

#include "types.h"

int {crate}_ffi_init(void);
const char* {crate}_ffi_version(void);
char* {crate}_ffi_process(const char* input);
void {crate}_ffi_free_string(char* s);

#endif
'''

def gen_c_ffi_c(crate):
    return f'''#include "ffi.h"
#include <stdlib.h>
#include <string.h>

int {crate}_ffi_init(void) {{
  return 0;
}}

const char* {crate}_ffi_version(void) {{
  return "{crate} 0.1.0";
}}

char* {crate}_ffi_process(const char* input) {{
  if (!input) return strdup("error: null input");
  return strdup("ok");
}}

void {crate}_ffi_free_string(char* s) {{
  free(s);
}}
'''

# ---------------------------------------------------------------------------
# 5. Bash scripts (5 files)
# ---------------------------------------------------------------------------
def gen_bash_build(crate):
    return f'''#!/usr/bin/env bash
set -euo pipefail

echo "Building {crate} bindings..."

SCRIPT_DIR="$(cd "$(dirname "${{BASH_SOURCE[0]}}")" && pwd)"
CRATE_DIR="$SCRIPT_DIR/../.."

cd "$CRATE_DIR"
cargo build --release 2>&1

echo "{crate} build complete"
'''

def gen_bash_test(crate):
    return f'''#!/usr/bin/env bash
set -euo pipefail

echo "Testing {crate} bindings..."

SCRIPT_DIR="$(cd "$(dirname "${{BASH_SOURCE[0]}}")" && pwd)"
CRATE_DIR="$SCRIPT_DIR/../.."

cd "$CRATE_DIR"
cargo test 2>&1

echo "{crate} tests complete"
'''

def gen_bash_bench(crate):
    return f'''#!/usr/bin/env bash
set -euo pipefail

echo "Benchmarking {crate} bindings..."

SCRIPT_DIR="$(cd "$(dirname "${{BASH_SOURCE[0]}}")" && pwd)"
CRATE_DIR="$SCRIPT_DIR/../.."

cd "$CRATE_DIR"
cargo bench 2>&1

echo "{crate} benchmarks complete"
'''

def gen_bash_dev(crate):
    return f'''#!/usr/bin/env bash
set -euo pipefail

echo "Starting {crate} dev mode..."

SCRIPT_DIR="$(cd "$(dirname "${{BASH_SOURCE[0]}}")" && pwd)"
CRATE_DIR="$SCRIPT_DIR/../.."

cd "$CRATE_DIR"
cargo watch -x check -x test 2>&1
'''

def gen_bash_clean(crate):
    return f'''#!/usr/bin/env bash
set -euo pipefail

echo "Cleaning {crate} bindings..."

SCRIPT_DIR="$(cd "$(dirname "${{BASH_SOURCE[0]}}")" && pwd)"
CRATE_DIR="$SCRIPT_DIR/../.."

cd "$CRATE_DIR"
cargo clean 2>&1

echo "{crate} clean complete"
'''

# ---------------------------------------------------------------------------
# Engine implementations
# ---------------------------------------------------------------------------
def gen_engine_lib(engine):
    cap_eng = engine.upper()
    return f'''//! Klyron JS engine — {engine}

pub mod runtime;
pub mod isolate;
pub mod module_loader;
pub mod bindings;
pub mod value;
pub mod promise;
pub mod error;
pub mod snapshot;
pub mod permissions;

pub use runtime::{engine}Runtime;
pub use isolate::{engine}Isolate;
pub use error::{engine}Error;
'''

def gen_engine_mod(engine):
    if engine == "v8":
        return f'''pub mod runtime;
pub mod isolate;
pub mod module_loader;
pub mod bindings;
pub mod value;
pub mod promise;
pub mod error;
pub mod snapshot;
pub mod permissions;
'''
    return f'''pub(crate) mod runtime;
pub(crate) mod isolate;
pub(crate) mod module_loader;
pub(crate) mod bindings;
pub(crate) mod value;
pub(crate) mod promise;
pub(crate) mod error;
pub(crate) mod snapshot;
pub(crate) mod permissions;
'''

def gen_engine_runtime(engine):
    cap = engine.upper()
    name = engine.capitalize()
    return f'''//! {name} runtime implementation

use crate::error::{engine}Error;

pub struct {name}Runtime {{
    pub isolate: crate::isolate::{name}Isolate,
    initialized: bool,
}}

impl {name}Runtime {{
    pub fn new() -> Self {{
        Self {{
            isolate: crate::isolate::{name}Isolate::new(),
            initialized: false,
        }}
    }}

    pub fn init(&mut self) -> Result<(), {engine}Error> {{
        self.initialized = true;
        Ok(())
    }}

    pub fn is_initialized(&self) -> bool {{
        self.initialized
    }}

    pub fn execute(&self, _code: &str) -> Result<String, {engine}Error> {{
        if !self.initialized {{
            return Err({engine}Error::NotInitialized);
        }}
        Ok(String::new())
    }}
}}

impl Default for {name}Runtime {{
    fn default() -> Self {{
        Self::new()
    }}
}}
'''

def gen_engine_isolate(engine):
    name = engine.capitalize()
    return f'''//! {name} isolate / context management

pub struct {name}Isolate {{
    pub context_created: bool,
}}

impl {name}Isolate {{
    pub fn new() -> Self {{
        Self {{ context_created: false }}
    }}

    pub fn create_context(&mut self) -> Result<(), crate::error::{engine}Error> {{
        self.context_created = true;
        Ok(())
    }}

    pub fn destroy_context(&mut self) {{
        self.context_created = false;
    }}
}}
'''

def gen_engine_module_loader(engine):
    name = engine.capitalize()
    return f'''//! Module loading for {name}

use crate::error::{engine}Error;

pub struct {name}ModuleLoader;

impl {name}ModuleLoader {{
    pub fn new() -> Self {{
        Self
    }}

    pub fn load(&self, _path: &str) -> Result<String, {engine}Error> {{
        Ok(String::new())
    }}

    pub fn resolve(&self, _specifier: &str, _base: &str) -> Result<String, {engine}Error> {{
        Ok(String::new())
    }}
}}
'''

def gen_engine_bindings(engine):
    name = engine.capitalize()
    return f'''//! JS bindings registration for {name}

pub fn register_bindings() -> Vec<&'static str> {{
    vec!["console", "timers", "fetch"]
}}

pub fn get_native_binding(_name: &str) -> Option<fn() -> String> {{
    None
}}
'''

def gen_engine_value(engine):
    name = engine.capitalize()
    return f'''//! JS value conversion for {name}

use crate::error::{engine}Error;

#[derive(Debug, Clone)]
pub enum {name}Value {{
    Null,
    Undefined,
    Boolean(bool),
    Integer(i64),
    Number(f64),
    String(String),
    Array(Vec<{name}Value>),
    Object(std::collections::HashMap<String, {name}Value>),
}}

impl {name}Value {{
    pub fn from_json(json: &serde_json::Value) -> Self {{
        match json {{
            serde_json::Value::Null => Self::Null,
            serde_json::Value::Bool(b) => Self::Boolean(*b),
            serde_json::Value::Number(n) => n.as_i64()
                .map(Self::Integer)
                .unwrap_or_else(|| Self::Number(n.as_f64().unwrap_or(0.0))),
            serde_json::Value::String(s) => Self::String(s.clone()),
            serde_json::Value::Array(arr) => Self::Array(arr.iter().map(Self::from_json).collect()),
            serde_json::Value::Object(obj) => Self::Object(
                obj.iter().map(|(k,v)| (k.clone(), Self::from_json(v))).collect()
            ),
        }}
    }}

    pub fn to_json(&self) -> serde_json::Value {{
        match self {{
            Self::Null => serde_json::Value::Null,
            Self::Undefined => serde_json::Value::Null,
            Self::Boolean(b) => serde_json::Value::Bool(*b),
            Self::Integer(i) => serde_json::json!(i),
            Self::Number(n) => serde_json::json!(n),
            Self::String(s) => serde_json::Value::String(s.clone()),
            Self::Array(arr) => serde_json::Value::Array(arr.iter().map(|v| v.to_json()).collect()),
            Self::Object(map) => serde_json::Value::Object(
                map.iter().map(|(k,v)| (k.clone(), v.to_json())).collect()
            ),
        }}
    }}

    pub fn is_truthy(&self) -> bool {{
        match self {{
            Self::Null | Self::Undefined => false,
            Self::Boolean(b) => *b,
            Self::Integer(i) => *i != 0,
            Self::Number(n) => *n != 0.0 && !n.is_nan(),
            Self::String(s) => !s.is_empty(),
            Self::Array(a) => !a.is_empty(),
            Self::Object(_) => true,
        }}
    }}
}}

impl From<String> for {name}Value {{
    fn from(s: String) -> Self {{
        Self::String(s)
    }}
}}

impl From<i64> for {name}Value {{
    fn from(i: i64) -> Self {{
        Self::Integer(i)
    }}
}}

impl From<f64> for {name}Value {{
    fn from(n: f64) -> Self {{
        Self::Number(n)
    }}
}}

impl From<bool> for {name}Value {{
    fn from(b: bool) -> Self {{
        Self::Boolean(b)
    }}
}}
'''

def gen_engine_promise(engine):
    name = engine.capitalize()
    return f'''//! Promise / future handling for {name}

use std::future::Future;
use std::pin::Pin;
use std::task::{{Context, Poll}};

pub struct {name}Promise<T> {{
    value: Option<T>,
}}

impl<T> {name}Promise<T> {{
    pub fn new(value: T) -> Self {{
        Self {{ value: Some(value) }}
    }}

    pub fn pending() -> Self {{
        Self {{ value: None }}
    }}

    pub fn resolve(&mut self, value: T) {{
        self.value = Some(value);
    }}

    pub fn take(&mut self) -> Option<T> {{
        self.value.take()
    }}
}}

impl<T: Unpin> Future for {name}Promise<T> {{
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {{
        let this = self.get_mut();
        if let Some(value) = this.value.take() {{
            Poll::Ready(value)
        }} else {{
            cx.waker().wake_by_ref();
            Poll::Pending
        }}
    }}
}}
'''

def gen_engine_error(engine):
    name = engine.capitalize()
    cap_eng = engine.upper()
    return f'''//! Error types for {name}

use std::fmt;

#[derive(Debug)]
pub enum {name}Error {{
    NotInitialized,
    ExecutionFailed(String),
    CompileError(String),
    SyntaxError(String),
    TypeError(String),
    RangeError(String),
    ReferenceError(String),
    Timeout,
    OutOfMemory,
}}

impl fmt::Display for {name}Error {{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {{
        match self {{
            Self::NotInitialized => write!(f, "{name} not initialized"),
            Self::ExecutionFailed(msg) => write!(f, "Execution failed: {{msg}}"),
            Self::CompileError(msg) => write!(f, "Compile error: {{msg}}"),
            Self::SyntaxError(msg) => write!(f, "Syntax error: {{msg}}"),
            Self::TypeError(msg) => write!(f, "Type error: {{msg}}"),
            Self::RangeError(msg) => write!(f, "Range error: {{msg}}"),
            Self::ReferenceError(msg) => write!(f, "Reference error: {{msg}}"),
            Self::Timeout => write!(f, "Script timeout"),
            Self::OutOfMemory => write!(f, "Out of memory"),
        }}
    }}
}}

impl std::error::Error for {name}Error {{}}

impl From<anyhow::Error> for {name}Error {{
    fn from(e: anyhow::Error) -> Self {{
        Self::ExecutionFailed(e.to_string())
    }}
}}
'''

def gen_engine_snapshot(engine):
    name = engine.capitalize()
    return f'''//! Snapshot support for {name}

use crate::error::{engine}Error;

pub struct {name}Snapshot {{
    data: Vec<u8>,
    created_at: std::time::SystemTime,
}}

impl {name}Snapshot {{
    pub fn new(data: Vec<u8>) -> Self {{
        Self {{ data, created_at: std::time::SystemTime::now() }}
    }}

    pub fn create(runtime: &crate::runtime::{name}Runtime) -> Result<Self, {engine}Error> {{
        if !runtime.is_initialized() {{
            return Err({engine}Error::NotInitialized);
        }}
        Ok(Self {{
            data: vec![],
            created_at: std::time::SystemTime::now(),
        }})
    }}

    pub fn load(data: &[u8]) -> Result<crate::runtime::{name}Runtime, {engine}Error> {{
        let runtime = crate::runtime::{name}Runtime::new();
        Ok(runtime)
    }}

    pub fn as_bytes(&self) -> &[u8] {{
        &self.data
    }}
}}
'''

def gen_engine_permissions(engine):
    name = engine.capitalize()
    return f'''//! Permission checking for {name}

#[derive(Debug, Clone)]
pub enum Permission {{
    Read,
    Write,
    Net,
    Env,
    Run,
    Ffi,
    All,
}}

#[derive(Debug, Default)]
pub struct {name}Permissions {{
    pub allow_read: Vec<String>,
    pub allow_write: Vec<String>,
    pub allow_net: Vec<String>,
    pub allow_env: bool,
    pub allow_run: bool,
    pub allow_ffi: bool,
}}

impl {name}Permissions {{
    pub fn new() -> Self {{
        Self::default()
    }}

    pub fn check(&self, permission: &Permission, resource: Option<&str>) -> bool {{
        match permission {{
            Permission::Read => self.check_path(&self.allow_read, resource),
            Permission::Write => self.check_path(&self.allow_write, resource),
            Permission::Net => self.check_net(resource),
            Permission::Env => self.allow_env,
            Permission::Run => self.allow_run,
            Permission::Ffi => self.allow_ffi,
            Permission::All => true,
        }}
    }}

    fn check_path(&self, allowed: &[String], resource: Option<&str>) -> bool {{
        if allowed.is_empty() {{ return false; }}
        if allowed.iter().any(|p| p == "/") {{ return true; }}
        if let Some(r) = resource {{
            allowed.iter().any(|p| r.starts_with(p))
        }} else {{
            false
        }}
    }}

    fn check_net(&self, resource: Option<&str>) -> bool {{
        if self.allow_net.is_empty() {{ return false; }}
        if self.allow_net.iter().any(|p| p == "*") {{ return true; }}
        if let Some(r) = resource {{
            self.allow_net.iter().any(|p| p == r)
        }} else {{
            false
        }}
    }}
}}
'''

def gen_engine_cargo_toml(engine):
    deps = {
        "v8": 'v8 = { version = "132", features = ["rusty_v8"] }',
        "boa": 'boa_engine = "0.20"',
        "quickjs": 'quickjs-wasm-sys = "0.4"',
        "jsc": 'javascriptcore-rs = "3.1"',
    }
    dep = deps.get(engine, "anyhow = \"1\"\nserde = { version = \"1\", features = [\"derive\"] }")
    eng_name = engine.capitalize()
    return f'''[package]
name = "klyron-engine-{engine}"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
{chr(9) if dep.startswith("anyhow") else ""}{dep}
anyhow = "1"
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"

[lib]
name = "klyron_engine_{engine}"
path = "src/lib.rs"
'''

# ---------------------------------------------------------------------------
# WRITE ALL FILES
# ---------------------------------------------------------------------------

# Rust bindings
def write_rust(crate):
    d = f"{ROOT}/crates/{crate}/bindings/rust"
    os.makedirs(d, exist_ok=True)
    files = {
        "types.rs": gen_rust_types,
        "errors.rs": gen_rust_errors,
        "builder.rs": gen_rust_builder,
        "config.rs": gen_rust_config,
        "ffi.rs": gen_rust_ffi,
        "wasm.rs": gen_rust_wasm,
        "client.rs": gen_rust_client,
        "server.rs": gen_rust_server,
        "test_utils.rs": gen_rust_test_utils,
        "lib.rs": gen_rust_lib,
    }
    for name, func in files.items():
        with open(f"{d}/{name}", "w") as f:
            f.write(func(crate))

# TypeScript bindings
def write_ts(crate):
    d = f"{ROOT}/crates/{crate}/bindings/ts"
    os.makedirs(d, exist_ok=True)
    files = {
        "types.ts": gen_ts_types,
        "client.ts": gen_ts_client,
        "errors.ts": gen_ts_errors,
        "config.ts": gen_ts_config,
        "utils.ts": gen_ts_utils,
        "index.ts": gen_ts_index,
        "test.ts": gen_ts_test,
        "benchmark.ts": gen_ts_benchmark,
        "fixtures.ts": gen_ts_fixtures,
        "declaration.d.ts": gen_ts_declaration,
    }
    for name, func in files.items():
        with open(f"{d}/{name}", "w") as f:
            f.write(func(crate))

# C++ bindings
def write_cpp(crate):
    d = f"{ROOT}/crates/{crate}/bindings/cpp"
    os.makedirs(d, exist_ok=True)
    files = {
        "types.hpp": gen_cpp_types_hpp,
        "types.cpp": gen_cpp_types_cpp,
        "api.hpp": gen_cpp_api_hpp,
        "api.cpp": gen_cpp_api_cpp,
        "config.hpp": gen_cpp_config_hpp,
        "config.cpp": gen_cpp_config_cpp,
        "errors.hpp": gen_cpp_errors_hpp,
        "errors.cpp": gen_cpp_errors_cpp,
        "builder.hpp": gen_cpp_builder_hpp,
        "builder.cpp": gen_cpp_builder_cpp,
        "ffi.hpp": gen_cpp_ffi_hpp,
    }
    for name, func in files.items():
        with open(f"{d}/{name}", "w") as f:
            f.write(func(crate))

# C bindings
def write_c(crate):
    d = f"{ROOT}/crates/{crate}/bindings/c"
    os.makedirs(d, exist_ok=True)
    files = {
        "types.h": gen_c_types_h,
        "types.c": gen_c_types_c,
        "api.h": gen_c_api_h,
        "api.c": gen_c_api_c,
        "config.h": gen_c_config_h,
        "config.c": gen_c_config_c,
        "errors.h": gen_c_errors_h,
        "errors.c": gen_c_errors_c,
        "ffi.h": gen_c_ffi_h,
        "ffi.c": gen_c_ffi_c,
    }
    for name, func in files.items():
        with open(f"{d}/{name}", "w") as f:
            f.write(func(crate))

# Bash scripts
def write_bash(crate):
    d = f"{ROOT}/crates/{crate}/bindings/bash"
    os.makedirs(d, exist_ok=True)
    files = {
        "build.sh": gen_bash_build,
        "test.sh": gen_bash_test,
        "bench.sh": gen_bash_bench,
        "dev.sh": gen_bash_dev,
        "clean.sh": gen_bash_clean,
    }
    for name, func in files.items():
        fp = f"{d}/{name}"
        with open(fp, "w") as f:
            f.write(func(crate))
        os.chmod(fp, 0o755)

# Engine files
def write_engine(engine):
    d = f"{ROOT}/engines/{engine}/src"
    os.makedirs(d, exist_ok=True)
    files = {
        "lib.rs": gen_engine_lib,
        "mod.rs": gen_engine_mod,
        "runtime.rs": gen_engine_runtime,
        "isolate.rs": gen_engine_isolate,
        "module_loader.rs": gen_engine_module_loader,
        "bindings.rs": gen_engine_bindings,
        "value.rs": gen_engine_value,
        "promise.rs": gen_engine_promise,
        "error.rs": gen_engine_error,
        "snapshot.rs": gen_engine_snapshot,
        "permissions.rs": gen_engine_permissions,
    }
    for name, func in files.items():
        with open(f"{d}/{name}", "w") as f:
            f.write(func(engine))

    # Cargo.toml
    with open(f"{ROOT}/engines/{engine}/Cargo.toml", "w") as f:
        f.write(gen_engine_cargo_toml(engine))


# ---------------------------------------------------------------------------
# MAIN
# ---------------------------------------------------------------------------
def main():
    total = 0
    for crate in CRATES:
        write_rust(crate); total += 10
        write_ts(crate); total += 10
        write_cpp(crate); total += 11
        write_c(crate); total += 10
        write_bash(crate); total += 5
        print(f"✓ {crate} — 46 binding files")

    for engine in ENGINES:
        write_engine(engine); total += 12  # 11 Rust + 1 Cargo.toml
        print(f"✓ engines/{engine} — 12 files (11 Rust + Cargo.toml)")

    print(f"\nTotal: {total} files created")

if __name__ == "__main__":
    main()
