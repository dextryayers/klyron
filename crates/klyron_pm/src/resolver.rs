use std::cmp::Reverse;
use std::collections::HashMap;

use pubgrub::{
    Dependencies, DependencyConstraints, DependencyProvider, DefaultStringReporter,
    PackageResolutionStatistics, PubGrubError, Ranges, Reporter,
};

use crate::{LockfileV3, PmError};

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
