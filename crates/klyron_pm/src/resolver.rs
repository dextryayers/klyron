use std::cmp::Reverse;
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

use pubgrub::{
    Dependencies, DependencyConstraints, DependencyProvider, DefaultStringReporter,
    PackageResolutionStatistics, PubGrubError, Ranges, Reporter,
};

use crate::{LockfileV3, PmError};

/// Metadata for a single published version: (dependencies, resolved tarball url).
pub struct VersionMeta {
    pub deps: HashMap<String, String>,
    pub resolved: String,
}

/// Full registry metadata for a package.
pub struct PackageMetadata {
    pub dist_tags: BTreeMap<String, String>,
    pub versions: BTreeMap<String, VersionMeta>,
}

/// Fetch full package metadata from the npm registry (all versions + their deps).
pub fn fetch_package_metadata(name: &str) -> Result<PackageMetadata, PmError> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build();
    let client = match client {
        Ok(c) => c,
        Err(e) => {
            return Err(PmError::IoError(format!("HTTP client: {}", e)));
        }
    };
    let url = format!("https://registry.npmjs.org/{}", name);

    // Retry transient network failures a few times before giving up.
    let mut last_err: Option<PmError> = None;
    for attempt in 0..4 {
        let req = client
            .get(&url)
            .header("Accept", "application/json")
            .build()
            .map_err(|e| PmError::IoError(format!("Build request: {}", e)))?;
        match client.execute(req) {
            Ok(resp) => {
                if resp.status().as_u16() == 404 {
                    return Err(PmError::PackageNotFound(name.to_string()));
                }
                if !resp.status().is_success() {
                    last_err = Some(PmError::IoError(format!(
                        "Registry {} status {}",
                        name,
                        resp.status()
                    )));
                    std::thread::sleep(std::time::Duration::from_millis(200 * (attempt + 1)));
                    continue;
                }
                let body: serde_json::Value = match resp.json() {
                    Ok(b) => b,
                    Err(e) => {
                        last_err = Some(PmError::IoError(format!("Parse metadata failed: {}", e)));
                        std::thread::sleep(std::time::Duration::from_millis(200 * (attempt + 1)));
                        continue;
                    }
                };
                return parse_metadata(name, body);
            }
            Err(e) => {
                last_err = Some(PmError::IoError(format!("Fetch {}: {}", name, e)));
                std::thread::sleep(std::time::Duration::from_millis(200 * (attempt + 1)));
            }
        }
    }
    Err(last_err.unwrap_or_else(|| PmError::PackageNotFound(name.to_string())))
}

fn parse_metadata(_name: &str, body: serde_json::Value) -> Result<PackageMetadata, PmError> {
    let mut dist_tags = BTreeMap::new();
    if let Some(tags) = body.get("dist-tags").and_then(|v| v.as_object()) {
        for (k, v) in tags {
            if let Some(s) = v.as_str() {
                dist_tags.insert(k.clone(), s.to_string());
            }
        }
    }

    let mut dist_tags = BTreeMap::new();
    if let Some(tags) = body.get("dist-tags").and_then(|v| v.as_object()) {
        for (k, v) in tags {
            if let Some(s) = v.as_str() {
                dist_tags.insert(k.clone(), s.to_string());
            }
        }
    }

    let mut versions = BTreeMap::new();
    if let Some(vers) = body.get("versions").and_then(|v| v.as_object()) {
        for (ver, meta) in vers {
            let deps = meta
                .get("dependencies")
                .and_then(|v| v.as_object())
                .map(|o| {
                    o.iter()
                        .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                        .collect::<HashMap<String, String>>()
                })
                .unwrap_or_default();
            let resolved = meta
                .get("dist")
                .and_then(|d| d.get("tarball"))
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .to_string();
            versions.insert(ver.clone(), VersionMeta { deps, resolved });
        }
    }

    Ok(PackageMetadata { dist_tags, versions })
}

/// Resolve a fresh dependency tree from a root dependency map (name -> version
/// range), fetching metadata from the registry on demand. This is what makes
/// `klyron` able to resolve dependencies WITHOUT delegating to npm.
pub fn resolve_fresh(
    root_name: &str,
    root_version: &str,
    root_deps: &HashMap<String, String>,
) -> Result<HashMap<String, semver::Version>, PmError> {
    let mut provider = KlyronDependencyProvider::new();
    let mut visited: HashSet<String> = HashSet::new();

    let root_ver = semver::Version::parse(root_version)
        .map_err(|e| PmError::ResolutionError(format!("invalid root version: {e}")))?;
    provider.add_package_version(root_name.to_string(), root_ver.clone());

    let mut root_constraints = DependencyConstraints::default();
    for (name, range) in root_deps {
        root_constraints.insert(name.clone(), normalize_range(range));
    }
    provider.add_dependencies(root_name.to_string(), root_ver.clone(), root_constraints);

    let mut queue: VecDeque<String> = root_deps.keys().cloned().collect();
    while let Some(pkg) = queue.pop_front() {
        if !visited.insert(pkg.clone()) {
            continue;
        }
        let meta = fetch_package_metadata(&pkg)?;
        for (ver, vm) in &meta.versions {
            if let Ok(v) = semver::Version::parse(ver) {
                provider.add_package_version(pkg.clone(), v.clone());
                let mut dc = DependencyConstraints::default();
                for (dn, dr) in &vm.deps {
                    dc.insert(dn.clone(), normalize_range(dr));
                    if !visited.contains(dn) && !queue.contains(dn) {
                        queue.push_back(dn.clone());
                    }
                }
                provider.add_dependencies(pkg.clone(), v, dc);
            }
        }
    }

    match pubgrub::resolve(&provider, root_name.to_string(), root_ver) {
        Ok(solution) => {
            let result: HashMap<String, semver::Version> = solution
                .into_iter()
                .filter(|(name, _)| name != root_name)
                .collect();
            Ok(result)
        }
        Err(PubGrubError::NoSolution(derivation_tree)) => {
            let report = DefaultStringReporter::report(&derivation_tree);
            Err(PmError::ResolutionError(format!(
                "Dependency resolution failed:\n{}",
                report
            )))
        }
        Err(err) => Err(PmError::ResolutionError(format!(
            "Dependency resolution failed: {:?}",
            err
        ))),
    }
}

/// Same as [`resolve_fresh`] but also returns the resolved tarball URL for each
/// package so the caller can download and extract it without involving npm.
pub fn resolve_fresh_install(
    root_name: &str,
    root_version: &str,
    root_deps: &HashMap<String, String>,
) -> Result<HashMap<String, (semver::Version, String)>, PmError> {
    let versions = resolve_fresh(root_name, root_version, root_deps)?;
    let mut out = HashMap::new();
    for (name, ver) in &versions {
        let meta = fetch_package_metadata(name)?;
        let key = ver.to_string();
        let url = meta
            .versions
            .get(&key)
            .and_then(|vm| {
                if vm.resolved.is_empty() {
                    None
                } else {
                    Some(vm.resolved.clone())
                }
            })
            .or_else(|| {
                Some(format!(
                    "https://registry.npmjs.org/{}/-/{}-{}.tgz",
                    name, name, ver
                ))
            });
        if let Some(u) = url {
            out.insert(name.clone(), (ver.clone(), u));
        }
    }
    Ok(out)
}

fn normalize_range(range: &str) -> Ranges<semver::Version> {
    if range.is_empty() || range == "*" {
        return Ranges::full();
    }
    let trimmed = range
        .trim_start_matches('^')
        .trim_start_matches('~')
        .trim_start_matches('=')
        .trim();
    match semver::VersionReq::parse(trimmed) {
        Ok(req) => Ranges::from_req(req),
        Err(_) => Ranges::full(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pubgrub::Ranges;
    use std::collections::HashMap;

    #[test]
    fn test_provider_new() {
        let provider = KlyronDependencyProvider::new();
        assert!(provider.packages.is_empty());
        assert!(provider.dependencies.is_empty());
    }

    #[test]
    fn test_normalize_range_full() {
        let r = normalize_range("*");
        // full range contains any version
        assert!(r.contains(&semver::Version::new(1, 0, 0)));
        assert!(r.contains(&semver::Version::new(99, 0, 0)));
    }

    #[test]
    #[ignore = "requires network access to npm registry"]
    fn test_fetch_package_metadata_express() {
        // Live registry fetch — proves klyron resolves WITHOUT npm.
        let meta = fetch_package_metadata("express").unwrap();
        assert!(!meta.versions.is_empty(), "express should have versions");
        assert!(meta.dist_tags.contains_key("latest"), "should have dist-tags.latest");
    }

    #[test]
    #[ignore = "requires network access to npm registry"]
    fn test_resolve_fresh_express() {
        // Resolve express fresh from the registry, no lockfile, no npm.
        let mut deps = HashMap::new();
        deps.insert("express".to_string(), "^4.18.0".to_string());
        let result = resolve_fresh("root", "1.0.0", &deps).unwrap();
        assert!(result.contains_key("express"), "express must be resolved");
        // express pulls in many deps (body-parser, send, etc.)
        assert!(result.len() > 1, "should resolve transitive deps");
    }

    #[test]
    fn test_add_package_version() {
        let mut provider = KlyronDependencyProvider::new();
        provider.add_package_version("foo".into(), semver::Version::new(1, 0, 0));
        assert_eq!(provider.packages.len(), 1);
        assert_eq!(provider.packages["foo"].len(), 1);
    }

    #[test]
    fn test_add_multiple_versions() {
        let mut provider = KlyronDependencyProvider::new();
        provider.add_package_version("foo".into(), semver::Version::new(1, 0, 0));
        provider.add_package_version("foo".into(), semver::Version::new(2, 0, 0));
        assert_eq!(provider.packages["foo"].len(), 2);
    }

    #[test]
    fn test_add_dependencies() {
        let mut provider = KlyronDependencyProvider::new();
        let ver = semver::Version::new(1, 0, 0);
        provider.add_package_version("foo".into(), ver.clone());
        let mut deps = DependencyConstraints::default();
        let range = Ranges::from_req(semver::VersionReq::parse(">=0.1.0").unwrap());
        deps.insert("bar".into(), range);
        provider.add_dependencies("foo".into(), ver.clone(), deps);
        assert!(provider.dependencies.contains_key(&("foo".into(), ver)));
    }

    #[test]
    fn test_choose_version_returns_highest() {
        let mut provider = KlyronDependencyProvider::new();
        provider.add_package_version("foo".into(), semver::Version::new(1, 0, 0));
        provider.add_package_version("foo".into(), semver::Version::new(2, 0, 0));
        let range = Ranges::full();
        let chosen = provider.choose_version(&"foo".into(), &range).unwrap();
        assert_eq!(chosen, Some(semver::Version::new(2, 0, 0)));
    }

    #[test]
    fn test_choose_version_no_match() {
        let provider = KlyronDependencyProvider::new();
        let range = Ranges::full();
        let chosen = provider.choose_version(&"nonexistent".into(), &range).unwrap();
        assert_eq!(chosen, None);
    }

    #[test]
    fn test_get_dependencies_unavailable() {
        let provider = KlyronDependencyProvider::new();
        let ver = semver::Version::new(1, 0, 0);
        let deps = provider.get_dependencies(&"foo".into(), &ver).unwrap();
        assert!(matches!(deps, Dependencies::Unavailable(_)));
    }

    #[test]
    fn test_prioritize_empty() {
        let provider = KlyronDependencyProvider::new();
        let range = Ranges::full();
        let stats = PackageResolutionStatistics::default();
        let priority = provider.prioritize(&"foo".into(), &range, &stats);
        // Should return some priority value
        assert_eq!(priority.1, Reverse(0));
    }

    #[test]
    fn test_resolve_from_lockfile_empty() {
        let lf = LockfileV3 {
            name: None, lockfile_version: None,
            packages: std::collections::BTreeMap::new(),
            workspaces: None, metadata: None,
        };
        let result = resolve_from_lockfile(&lf, "root", semver::Version::new(1, 0, 0));
        // Should fail since there are no dependencies registered
        assert!(result.is_err());
    }

    #[test]
    fn test_version_range_construction() {
        let req = semver::VersionReq::parse("^1.0.0").unwrap();
        let range = Ranges::from_req(req);
        let full = Ranges::full();
        assert_ne!(range, full);
    }
}

pub struct KlyronDependencyProvider {
    packages: HashMap<String, Vec<semver::Version>>,
    dependencies:
        HashMap<(String, semver::Version), DependencyConstraints<String, Ranges<semver::Version>>>,
}

impl KlyronDependencyProvider {
    pub fn new() -> Self {
        Self {
            packages: HashMap::new(),
            dependencies: HashMap::new(),
        }
    }

    pub fn from_lockfile(lockfile: &LockfileV3) -> Self {
        let mut provider = Self::new();
        for (path, pkg) in &lockfile.packages {
            let name = path.rsplit('/').next().unwrap_or(path);
            if let Ok(ver) = semver::Version::parse(&pkg.version) {
                provider.add_package_version(name.to_string(), ver);
            }
        }
        provider
    }

    pub fn add_package_version(&mut self, name: String, version: semver::Version) {
        self.packages.entry(name).or_default().push(version);
    }

    pub fn add_dependencies(
        &mut self,
        package: String,
        version: semver::Version,
        deps: DependencyConstraints<String, Ranges<semver::Version>>,
    ) {
        self.dependencies.insert((package, version), deps);
    }
}

impl DependencyProvider for KlyronDependencyProvider {
    type P = String;
    type V = semver::Version;
    type VS = Ranges<semver::Version>;
    type M = String;
    type Err = std::convert::Infallible;

    type Priority = (u32, Reverse<usize>);

    fn prioritize(
        &self,
        package: &String,
        range: &Ranges<semver::Version>,
        conflict_counts: &PackageResolutionStatistics,
    ) -> Self::Priority {
        let count = self
            .packages
            .get(package)
            .map(|versions| versions.iter().filter(|v| range.contains(v)).count())
            .unwrap_or(0);
        (u32::MAX - conflict_counts.conflict_count(), Reverse(count))
    }

    fn choose_version(
        &self,
        package: &String,
        range: &Ranges<semver::Version>,
    ) -> Result<Option<semver::Version>, std::convert::Infallible> {
        Ok(self
            .packages
            .get(package)
            .and_then(|versions| versions.iter().filter(|v| range.contains(v)).max().cloned()))
    }

    fn get_dependencies(
        &self,
        package: &String,
        version: &semver::Version,
    ) -> Result<
        Dependencies<String, Ranges<semver::Version>, String>,
        std::convert::Infallible,
    > {
        let deps = self.dependencies.get(&(package.clone(), version.clone()));
        match deps {
            None => Ok(Dependencies::Unavailable(
                "dependencies not registered".to_string(),
            )),
            Some(deps) if deps.is_empty() => {
                Ok(Dependencies::Available(DependencyConstraints::default()))
            }
            Some(deps) => Ok(Dependencies::Available(deps.clone())),
        }
    }
}

pub fn resolve_from_lockfile(
    lockfile: &LockfileV3,
    root_name: &str,
    root_version: semver::Version,
) -> Result<HashMap<String, semver::Version>, PmError> {
    let mut provider = KlyronDependencyProvider::from_lockfile(lockfile);

    let root_deps: DependencyConstraints<String, Ranges<semver::Version>> = lockfile
        .packages
        .iter()
        .filter(|(_, pkg)| pkg.dependencies.is_some())
        .flat_map(|(_, pkg)| {
            pkg.dependencies
                .as_ref()
                .unwrap()
                .iter()
                .filter_map(|(name, req)| {
                    let range = if req == "*" {
                        Ranges::full()
                    } else {
                        let semver_req = semver::VersionReq::parse(req).ok()?;
                        Ranges::from_req(semver_req)
                    };
                    Some((name.clone(), range))
                })
        })
        .collect();

    provider.add_dependencies(root_name.to_string(), root_version.clone(), root_deps);

    match pubgrub::resolve(&provider, root_name.to_string(), root_version) {
        Ok(solution) => {
            let result: HashMap<String, semver::Version> = solution
                .into_iter()
                .filter(|(name, _)| name != root_name)
                .collect();
            Ok(result)
        }
        Err(PubGrubError::NoSolution(derivation_tree)) => {
            let report = DefaultStringReporter::report(&derivation_tree);
            Err(PmError::ResolutionError(format!(
                "Dependency resolution failed:\n{}",
                report
            )))
        }
        Err(err) => Err(PmError::ResolutionError(format!(
            "Dependency resolution failed: {:?}",
            err
        ))),
    }
}
