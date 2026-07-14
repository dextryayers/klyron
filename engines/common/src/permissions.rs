#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommonPermission {
    Read,
    Write,
    Net,
    Env,
    Run,
    Ffi,
    All,
}

#[derive(Debug, Clone)]
pub struct CommonPermissions {
    pub allow_read: Vec<String>,
    pub allow_write: Vec<String>,
    pub allow_net: Vec<String>,
    pub allow_env: bool,
    pub allow_run: bool,
    pub allow_ffi: bool,
}

impl CommonPermissions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn check(&self, permission: &CommonPermission, resource: Option<&str>) -> bool {
        match permission {
            CommonPermission::Read => self.check_path(&self.allow_read, resource),
            CommonPermission::Write => self.check_path(&self.allow_write, resource),
            CommonPermission::Net => self.check_net(resource),
            CommonPermission::Env => self.allow_env,
            CommonPermission::Run => self.allow_run,
            CommonPermission::Ffi => self.allow_ffi,
            CommonPermission::All => true,
        }
    }

    pub fn deny_all() -> Self {
        Self::default()
    }

    pub fn allow_all() -> Self {
        Self {
            allow_read: vec!["/".to_string()],
            allow_write: vec!["/".to_string()],
            allow_net: vec!["*".to_string()],
            allow_env: true,
            allow_run: true,
            allow_ffi: true,
        }
    }

    fn check_path(&self, allowed: &[String], resource: Option<&str>) -> bool {
        if allowed.is_empty() {
            return false;
        }
        if allowed.iter().any(|p| p == "/") {
            return true;
        }
        if let Some(r) = resource {
            allowed.iter().any(|p| r.starts_with(p))
        } else {
            false
        }
    }

    fn check_net(&self, resource: Option<&str>) -> bool {
        if self.allow_net.is_empty() {
            return false;
        }
        if self.allow_net.iter().any(|p| p == "*") {
            return true;
        }
        if let Some(r) = resource {
            self.allow_net.iter().any(|p| p == r)
        } else {
            false
        }
    }
}

impl Default for CommonPermissions {
    fn default() -> Self {
        Self {
            allow_read: Vec::new(),
            allow_write: Vec::new(),
            allow_net: Vec::new(),
            allow_env: false,
            allow_run: false,
            allow_ffi: false,
        }
    }
}
