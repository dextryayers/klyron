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

    pub fn check(&self, permission: &Permission, resource: Option<&str>) -> bool {
        let common: CommonPermission = permission.clone().into();
        let common_perms = self.to_common();
        common_perms.check(&common, resource)
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
