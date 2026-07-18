use klyron_engine_common::permissions::{CommonPermission, CommonPermissions};

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

impl From<Permission> for CommonPermission {
    fn from(p: Permission) -> Self {
        match p {
            Permission::Read => CommonPermission::Read,
            Permission::Write => CommonPermission::Write,
            Permission::Net => CommonPermission::Net,
            Permission::Env => CommonPermission::Env,
            Permission::Run => CommonPermission::Run,
            Permission::Ffi => CommonPermission::Ffi,
            Permission::All => CommonPermission::All,
        }
    }
}

impl From<CommonPermission> for Permission {
    fn from(p: CommonPermission) -> Self {
        match p {
            CommonPermission::Read => Permission::Read,
            CommonPermission::Write => Permission::Write,
            CommonPermission::Net => Permission::Net,
            CommonPermission::Env => Permission::Env,
            CommonPermission::Run => Permission::Run,
            CommonPermission::Ffi => Permission::Ffi,
            CommonPermission::All => Permission::All,
        }
    }
}

#[derive(Debug, Default)]
pub struct BoaPermissions {
    pub allow_read: Vec<String>,
    pub allow_write: Vec<String>,
    pub allow_net: Vec<String>,
    pub allow_env: bool,
    pub allow_run: bool,
    pub allow_ffi: bool,
}

impl BoaPermissions {
    pub fn new() -> Self {
        Self::default()
    }

    fn check_path(allowed: &[String], resource: Option<&str>) -> bool {
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

    fn check_net(allowed: &[String], resource: Option<&str>) -> bool {
        if allowed.is_empty() {
            return false;
        }
        if allowed.iter().any(|p| p == "*") {
            return true;
        }
        if let Some(r) = resource {
            allowed.iter().any(|p| p == r)
        } else {
            false
        }
    }

    pub fn check(&self, permission: &Permission, resource: Option<&str>) -> bool {
        match permission {
            Permission::Read => Self::check_path(&self.allow_read, resource),
            Permission::Write => Self::check_path(&self.allow_write, resource),
            Permission::Net => Self::check_net(&self.allow_net, resource),
            Permission::Env => self.allow_env,
            Permission::Run => self.allow_run,
            Permission::Ffi => self.allow_ffi,
            Permission::All => true,
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

    pub fn to_common(&self) -> CommonPermissions {
        CommonPermissions {
            allow_read: self.allow_read.clone(),
            allow_write: self.allow_write.clone(),
            allow_net: self.allow_net.clone(),
            allow_env: self.allow_env,
            allow_run: self.allow_run,
            allow_ffi: self.allow_ffi,
        }
    }

    pub fn from_common(common: CommonPermissions) -> Self {
        Self {
            allow_read: common.allow_read,
            allow_write: common.allow_write,
            allow_net: common.allow_net,
            allow_env: common.allow_env,
            allow_run: common.allow_run,
            allow_ffi: common.allow_ffi,
        }
    }

}
