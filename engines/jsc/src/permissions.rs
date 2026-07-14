//! Permission checking for Jsc

#[derive(Debug, Clone)]
pub enum Permission {
    Read,
    Write,
    Net,
    Env,
    Run,
    Ffi,
    All,
}

#[derive(Debug, Default)]
pub struct JscPermissions {
    pub allow_read: Vec<String>,
    pub allow_write: Vec<String>,
    pub allow_net: Vec<String>,
    pub allow_env: bool,
    pub allow_run: bool,
    pub allow_ffi: bool,
}

impl JscPermissions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn check(&self, permission: &Permission, resource: Option<&str>) -> bool {
        match permission {
            Permission::Read => self.check_path(&self.allow_read, resource),
            Permission::Write => self.check_path(&self.allow_write, resource),
            Permission::Net => self.check_net(resource),
            Permission::Env => self.allow_env,
            Permission::Run => self.allow_run,
            Permission::Ffi => self.allow_ffi,
            Permission::All => true,
        }
    }

    fn check_path(&self, allowed: &[String], resource: Option<&str>) -> bool {
        if allowed.is_empty() { return false; }
        if allowed.iter().any(|p| p == "/") { return true; }
        if let Some(r) = resource {
            allowed.iter().any(|p| r.starts_with(p))
        } else {
            false
        }
    }

    fn check_net(&self, resource: Option<&str>) -> bool {
        if self.allow_net.is_empty() { return false; }
        if self.allow_net.iter().any(|p| p == "*") { return true; }
        if let Some(r) = resource {
            self.allow_net.iter().any(|p| p == r)
        } else {
            false
        }
    }
}
