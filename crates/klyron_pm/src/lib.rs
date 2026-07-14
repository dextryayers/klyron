use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageManager {
    Npm,
    Yarn,
    Pnpm,
    Bun,
    Composer,
    Cargo,
    Go,
    Pip,
    Gem,
}

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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_detect_unknown() {
        let dir = std::path::Path::new("/tmp");
        assert_eq!(detect(dir), PackageManager::Npm);
    }
}
