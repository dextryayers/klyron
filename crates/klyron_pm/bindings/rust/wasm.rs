use wasm_bindgen::prelude::*;
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
