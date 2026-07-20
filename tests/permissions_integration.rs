#[cfg(test)]
mod tests {
    use klyron_core::permissions::PermissionSet;

    #[test]
    fn test_permission_set_default() {
        let p = PermissionSet::default();
        assert!(!p.allow_net_all);
        assert!(!p.allow_read_all);
    }

    #[test]
    fn test_permission_set_deny_read() {
        let mut p = PermissionSet::default();
        p.deny_read.push("/etc/passwd".into());
        assert!(p.deny_read.contains(&"/etc/passwd".to_string()));
    }

    #[test]
    fn test_permission_set_allow_write() {
        let mut p = PermissionSet::default();
        p.allow_write.push("/tmp".into());
        assert!(p.allow_write.contains(&"/tmp".to_string()));
    }
}
