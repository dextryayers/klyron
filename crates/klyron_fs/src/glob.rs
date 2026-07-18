use std::path::{Path, PathBuf};

use crate::FileInfo;

pub struct GlobPattern {
    pattern: String,
    case_insensitive: bool,
}

impl GlobPattern {
    pub fn new(pattern: &str) -> Self {
        Self {
            pattern: pattern.to_string(),
            case_insensitive: false,
        }
    }

    pub fn case_insensitive(mut self) -> Self {
        self.case_insensitive = true;
        self
    }

    pub fn matches(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        let pattern = if self.case_insensitive {
            self.pattern.to_lowercase()
        } else {
            self.pattern.clone()
        };
        let target = if self.case_insensitive {
            path_str.to_lowercase()
        } else {
            path_str.to_string()
        };

        glob::Pattern::new(&pattern)
            .map(|p| p.matches(&target))
            .unwrap_or(false)
    }
}

pub struct GlobBuilder {
    pattern: String,
    base_dir: Option<PathBuf>,
    recursive: bool,
    case_insensitive: bool,
}

impl GlobBuilder {
    pub fn new(pattern: &str) -> Self {
        Self {
            pattern: pattern.to_string(),
            base_dir: None,
            recursive: true,
            case_insensitive: false,
        }
    }

    pub fn base_dir(mut self, dir: &Path) -> Self {
        self.base_dir = Some(dir.to_path_buf());
        self
    }

    pub fn recursive(mut self, recursive: bool) -> Self {
        self.recursive = recursive;
        self
    }

    pub fn case_insensitive(mut self) -> Self {
        self.case_insensitive = true;
        self
    }

    pub fn build(self) -> anyhow::Result<Glob> {
        let base_dir = self.base_dir.unwrap_or_else(|| PathBuf::from("."));
        if !base_dir.exists() {
            anyhow::bail!("Base directory does not exist: {}", base_dir.display());
        }

        Ok(Glob {
            pattern: self.pattern,
            base_dir,
            recursive: self.recursive,
            case_insensitive: self.case_insensitive,
        })
    }
}

pub struct Glob {
    pattern: String,
    base_dir: PathBuf,
    recursive: bool,
    case_insensitive: bool,
}

impl Glob {
    pub fn paths(&self) -> anyhow::Result<Vec<PathBuf>> {
        let pattern = GlobPattern::new(&self.pattern);
        let pattern = if self.case_insensitive {
            pattern.case_insensitive()
        } else {
            pattern
        };

        let mut matches = Vec::new();
        let walker = walkdir::WalkDir::new(&self.base_dir);
        let walker = if self.recursive {
            walker.into_iter()
        } else {
            walker.max_depth(1).into_iter()
        };

        for entry in walker.filter_entry(|e| !e.file_name().to_string_lossy().starts_with('.')) {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && pattern.matches(path) {
                matches.push(path.to_path_buf());
            }
        }

        matches.sort();
        Ok(matches)
    }

    pub fn files(&self) -> anyhow::Result<Vec<FileInfo>> {
        let paths = self.paths()?;
        let fs = crate::FileSystem::new();
        paths.iter().map(|p| fs.stat(p)).collect()
    }

    pub fn count(&self) -> anyhow::Result<usize> {
        Ok(self.paths()?.len())
    }
}

pub fn glob(pattern: &str) -> anyhow::Result<Vec<PathBuf>> {
    GlobBuilder::new(pattern).build()?.paths()
}

pub fn glob_files(pattern: &str) -> anyhow::Result<Vec<FileInfo>> {
    GlobBuilder::new(pattern).build()?.files()
}
