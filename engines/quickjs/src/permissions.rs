use klyron_engine_common::permissions::{CommonPermission, CommonPermissions};

#[derive(Debug, Clone)]
pub enum Permission {
    Read, Write, Net, Env, Run, Ffi, All,
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
pub struct QuickJSPermissions(CommonPermissions);

impl QuickJSPermissions {
    pub fn new() -> Self { Self::default() }

    pub fn check(&self, permission: &Permission) -> bool {
        let common: CommonPermission = permission.clone().into();
        self.0.check(&common, None)
    }

    pub fn check_path(&self, permission: &Permission, path: &str) -> bool {
        let common: CommonPermission = permission.clone().into();
        self.0.check(&common, Some(path))
    }

    pub fn deny_all() -> Self { Self(CommonPermissions::deny_all()) }
    pub fn allow_all() -> Self { Self(CommonPermissions::allow_all()) }

    pub fn to_common(&self) -> &CommonPermissions { &self.0 }
    pub fn from_common(common: CommonPermissions) -> Self { Self(common) }
}
