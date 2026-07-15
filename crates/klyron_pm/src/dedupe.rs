use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyTree {
    pub name: String,
    pub version: String,
    pub dependencies: Vec<DependencyTree>,
    pub resolved: bool,
    pub integrity: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateInfo {
    pub name: String,
    pub versions: Vec<String>,
    pub locations: Vec<String>,
    pub potential_savings: u64,
    pub installed_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DedupeReport {
    pub original_size: u64,
    pub deduped_size: u64,
    pub savings_bytes: u64,
    pub savings_percent: f64,
    pub duplicates_found: usize,
    pub duplicates_resolved: usize,
    pub duplicate_details: Vec<DuplicateInfo>,
    pub hoist_opportunities: usize,
    pub hoist_savings: u64,
}

pub struct DedupeEngine;

impl DedupeEngine {
    pub fn dedupe_tree(tree: &DependencyTree) -> DependencyTree {
        let mut occurrences: HashMap<String, Vec<String>> = HashMap::new();
        collect_occurrences(tree, &mut occurrences);

        let mut best_versions: HashMap<String, String> = HashMap::new();
        for (name, versions) in &occurrences {
            if let Some(highest) = Self::highest_version(versions) {
                best_versions.insert(name.clone(), highest);
            }
        }

        let mut visited = HashSet::new();
        rebuild_deduped(tree, &best_versions, &mut visited)
    }

    pub fn hoist_deps(tree: &DependencyTree) -> DependencyTree {
        let mut result = tree.clone();
        perform_hoist(&mut result, &mut HashSet::new());
        result
    }

    pub fn find_duplicates(tree: &DependencyTree) -> Vec<DuplicateInfo> {
        let flat = Self::flatten_tree(tree, 0);
        let mut by_name: HashMap<String, Vec<(String, usize)>> = HashMap::new();

        for (name, version, depth) in flat {
            by_name.entry(name).or_default().push((version, depth));
        }

        let mut duplicates = Vec::new();
        for (name, entries) in by_name {
            if entries.len() > 1 {
                let versions: Vec<String> = entries.iter().map(|(v, _)| v.clone()).collect();
                let unique: HashSet<&str> = versions.iter().map(|s| s.as_str()).collect();
                if unique.len() > 1 {
                    let locations: Vec<String> = entries
                        .iter()
                        .enumerate()
                        .map(|(i, (v, d))| format!("[{}] {}@depth={}", i, v, d))
                        .collect();
                    let savings = Self::estimate_package_size(&name, &versions[0])
                        * (entries.len() - 1) as u64;
                    duplicates.push(DuplicateInfo {
                        name,
                        versions,
                        locations,
                        potential_savings: savings,
                        installed_count: entries.len(),
                    });
                }
            }
        }

        duplicates
    }

    pub fn analyze_savings(tree: &DependencyTree) -> DedupeReport {
        let duplicates = Self::find_duplicates(tree);
        let flat = Self::flatten_tree(tree, 0);
        let total_original: u64 = flat
            .iter()
            .map(|(name, version, _)| Self::estimate_package_size(name, version))
            .sum();

        let savings: u64 = duplicates.iter().map(|d| d.potential_savings).sum();
        let deduped_size = total_original.saturating_sub(savings);

        let hoist_opps = duplicates.iter().filter(|d| d.versions.len() > 1).count();
        let hoist_savings = hoist_opps as u64 * 10240;

        DedupeReport {
            original_size: total_original,
            deduped_size,
            savings_bytes: savings,
            savings_percent: if total_original > 0 {
                (savings as f64 / total_original as f64) * 100.0
            } else {
                0.0
            },
            duplicates_found: duplicates.len(),
            duplicates_resolved: 0,
            duplicate_details: duplicates,
            hoist_opportunities: hoist_opps,
            hoist_savings,
        }
    }

    pub fn generate_report(report: &DedupeReport, format: ReportFormat) -> String {
        match format {
            ReportFormat::Json => {
                serde_json::to_string_pretty(report).unwrap_or_default()
            }
            ReportFormat::Pretty => {
                let mut out = String::new();
                out.push_str("Deduplication Report\n");
                out.push_str(&format!("  Original size:     {} bytes\n", report.original_size));
                out.push_str(&format!("  Deduplicated size: {} bytes\n", report.deduped_size));
                out.push_str(&format!(
                    "  Savings:           {} bytes ({:.1}%)\n",
                    report.savings_bytes, report.savings_percent
                ));
                out.push_str(&format!("  Duplicates found:  {}\n", report.duplicates_found));
                out.push_str(&format!("  Duplicates resolved: {}\n", report.duplicates_resolved));
                out.push_str(&format!(
                    "  Hoist opportunities: {}\n",
                    report.hoist_opportunities
                ));
                out.push_str(&format!("  Hoist savings:     {} bytes\n", report.hoist_savings));
                if !report.duplicate_details.is_empty() {
                    out.push_str("\n  Duplicate Details:\n");
                    for dup in &report.duplicate_details {
                        out.push_str(&format!(
                            "    - {} ({}x): versions [{}]\n",
                            dup.name,
                            dup.installed_count,
                            dup.versions.join(", ")
                        ));
                        for loc in &dup.locations {
                            out.push_str(&format!("        {loc}\n"));
                        }
                    }
                }
                out
            }
            ReportFormat::Text => {
                format!(
                    "size:{} -> {} (saved {} bytes, {:.1}%), duplicates: {} found/{} resolved, hoist: {} opportunities/{} bytes",
                    report.original_size,
                    report.deduped_size,
                    report.savings_bytes,
                    report.savings_percent,
                    report.duplicates_found,
                    report.duplicates_resolved,
                    report.hoist_opportunities,
                    report.hoist_savings,
                )
            }
        }
    }

    fn flatten_tree(tree: &DependencyTree, depth: usize) -> Vec<(String, String, usize)> {
        let mut result = vec![(tree.name.clone(), tree.version.clone(), depth)];
        for child in &tree.dependencies {
            result.extend(Self::flatten_tree(child, depth + 1));
        }
        result
    }

    fn satisfies(version_a: &str, version_b: &str) -> bool {
        if let Ok(ver) = semver::Version::parse(version_a) {
            if let Ok(req) = semver::VersionReq::parse(version_b) {
                return req.matches(&ver);
            }
            if let Ok(other) = semver::Version::parse(version_b) {
                return ver == other;
            }
        }
        version_a == version_b
    }

    fn highest_version(versions: &[String]) -> Option<String> {
        versions
            .iter()
            .filter_map(|v| semver::Version::parse(v).ok().map(|sv| (v.clone(), sv)))
            .max_by(|(_, a), (_, b)| a.cmp(b))
            .map(|(v, _)| v)
            .or_else(|| versions.first().cloned())
    }

    fn estimate_package_size(name: &str, version: &str) -> u64 {
        let known_sizes: HashMap<&str, u64> = [
            ("react", 50),
            ("react-dom", 130),
            ("lodash", 550),
            ("axios", 90),
            ("express", 60),
            ("typescript", 6000),
            ("webpack", 1500),
            ("next", 4000),
            ("vue", 300),
            ("angular", 800),
            ("jquery", 30),
            ("moment", 250),
            ("date-fns", 130),
            ("uuid", 10),
            ("chalk", 40),
            ("commander", 30),
            ("yargs", 80),
            ("inquirer", 60),
            ("prettier", 500),
            ("eslint", 700),
        ]
        .iter()
        .map(|(k, v)| (*k, *v * 1024))
        .collect();

        let base = known_sizes.get(name).copied().unwrap_or(100 * 1024);

        if let Ok(ver) = semver::Version::parse(version) {
            (base as f64 * (1.0 + ver.major as f64 * 0.1)) as u64
        } else {
            base
        }
    }
}

#[derive(Debug, Clone)]
pub enum ReportFormat {
    Text,
    Json,
    Pretty,
}

pub mod semver_utils {
    pub fn parse_version(version: &str) -> Option<(u64, u64, u64, String)> {
        let v = semver::Version::parse(version).ok()?;
        let pre = v.pre.to_string();
        Some((v.major, v.minor, v.patch, pre))
    }

    pub fn satisfies(version: &str, range: &str) -> bool {
        if let Ok(ver) = semver::Version::parse(version) {
            if let Ok(req) = semver::VersionReq::parse(range) {
                return req.matches(&ver);
            }
            if let Ok(other) = semver::Version::parse(range) {
                return ver == other;
            }
        }
        version == range
    }

    pub fn max_satisfying(versions: &[String], range: &str) -> Option<String> {
        let req = semver::VersionReq::parse(range).ok()?;
        versions
            .iter()
            .filter_map(|v| semver::Version::parse(v).ok().map(|sv| (v.clone(), sv)))
            .filter(|(_, v)| req.matches(v))
            .max_by(|(_, a), (_, b)| a.cmp(b))
            .map(|(v, _)| v)
    }

    pub fn sort_versions(versions: &mut [String]) {
        versions.sort_by(|a, b| match (semver::Version::parse(a), semver::Version::parse(b)) {
            (Ok(va), Ok(vb)) => va.cmp(&vb),
            _ => a.cmp(b),
        });
    }
}

pub fn build_dependency_tree(
    root_name: &str,
    root_version: &str,
    packages: &HashMap<String, HashMap<String, String>>,
    versions: &HashMap<(String, String), Vec<(String, String)>>,
) -> DependencyTree {
    let mut visited = HashSet::new();
    build_node(root_name, root_version, packages, versions, &mut visited)
}

fn build_node(
    name: &str,
    version: &str,
    packages: &HashMap<String, HashMap<String, String>>,
    versions: &HashMap<(String, String), Vec<(String, String)>>,
    visited: &mut HashSet<(String, String)>,
) -> DependencyTree {
    let key = (name.to_string(), version.to_string());
    let is_cycle = !visited.insert(key.clone());

    let dependencies = if is_cycle {
        Vec::new()
    } else if let Some(dep_vec) = versions.get(&(name.to_string(), version.to_string())) {
        dep_vec
            .iter()
            .map(|(dep_name, dep_version)| {
                build_node(dep_name, dep_version, packages, versions, visited)
            })
            .collect()
    } else if let Some(dep_map) = packages.get(name) {
        dep_map
            .iter()
            .map(|(dep_name, dep_version)| {
                let resolved = resolve_version(dep_name, dep_version, versions);
                build_node(dep_name, &resolved, packages, versions, visited)
            })
            .collect()
    } else {
        Vec::new()
    };

    DependencyTree {
        name: name.to_string(),
        version: version.to_string(),
        dependencies,
        resolved: true,
        integrity: None,
    }
}

fn resolve_version(
    name: &str,
    version_req: &str,
    versions: &HashMap<(String, String), Vec<(String, String)>>,
) -> String {
    let candidates: Vec<String> = versions
        .keys()
        .filter(|(n, _)| n == name)
        .map(|(_, v)| v.clone())
        .collect();

    semver_utils::max_satisfying(&candidates, version_req)
        .unwrap_or_else(|| version_req.to_string())
}

fn collect_occurrences(tree: &DependencyTree, acc: &mut HashMap<String, Vec<String>>) {
    acc.entry(tree.name.clone())
        .or_default()
        .push(tree.version.clone());
    for child in &tree.dependencies {
        collect_occurrences(child, acc);
    }
}

fn rebuild_deduped(
    tree: &DependencyTree,
    best_versions: &HashMap<String, String>,
    visited: &mut HashSet<(String, String)>,
) -> DependencyTree {
    let key = (tree.name.clone(), tree.version.clone());
    if !visited.insert(key) {
        return DependencyTree {
            name: tree.name.clone(),
            version: tree.version.clone(),
            dependencies: Vec::new(),
            resolved: tree.resolved,
            integrity: tree.integrity.clone(),
        };
    }

    let version = best_versions
        .get(&tree.name)
        .cloned()
        .unwrap_or_else(|| tree.version.clone());

    let dependencies: Vec<DependencyTree> = tree
        .dependencies
        .iter()
        .map(|dep| rebuild_deduped(dep, best_versions, visited))
        .collect();

    DependencyTree {
        name: tree.name.clone(),
        version,
        dependencies,
        resolved: tree.resolved,
        integrity: tree.integrity.clone(),
    }
}

fn perform_hoist(node: &mut DependencyTree, visited: &mut HashSet<(String, String)>) {
    let key = (node.name.clone(), node.version.clone());
    if !visited.insert(key) {
        return;
    }

    for i in 0..node.dependencies.len() {
        perform_hoist(&mut node.dependencies[i], visited);
    }

    let mut child_dep_counts: HashMap<String, Vec<usize>> = HashMap::new();
    for (ci, child) in node.dependencies.iter().enumerate() {
        for dep in &child.dependencies {
            child_dep_counts
                .entry(dep.name.clone())
                .or_default()
                .push(ci);
        }
    }

    let to_hoist: Vec<(String, String)> = child_dep_counts
        .into_iter()
        .filter(|(_, indices)| indices.len() > 1)
        .filter_map(|(dep_name, indices)| {
            let versions: HashSet<&str> = indices
                .iter()
                .filter_map(|&ci| {
                    node.dependencies[ci]
                        .dependencies
                        .iter()
                        .find(|d| d.name == dep_name)
                        .map(|d| d.version.as_str())
                })
                .collect();
            if versions.len() == 1 {
                Some((dep_name, versions.into_iter().next().unwrap().to_string()))
            } else {
                None
            }
        })
        .collect();

    for (dep_name, dep_version) in &to_hoist {
        if !node
            .dependencies
            .iter()
            .any(|d| d.name == *dep_name && d.version == *dep_version)
        {
            node.dependencies.push(DependencyTree {
                name: dep_name.clone(),
                version: dep_version.clone(),
                dependencies: Vec::new(),
                resolved: true,
                integrity: None,
            });
        }
        for child in &mut node.dependencies {
            child
                .dependencies
                .retain(|d| !(d.name == *dep_name && d.version == *dep_version));
        }
    }
}
