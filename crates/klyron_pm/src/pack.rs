use crate::PmError;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tar::Builder as TarBuilder;

/// Configuration for packing a package
#[derive(Debug, Clone)]
pub struct PackConfig {
    pub root: PathBuf,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub sign: bool,
    pub signing_key: Option<Vec<u8>>,
}

impl Default for PackConfig {
    fn default() -> Self {
        Self {
            root: PathBuf::from("."),
            include: Vec::new(),
            exclude: vec![".git".into(), "node_modules".into(), "target".into()],
            sign: false,
            signing_key: None,
        }
    }
}

/// Pack a package into a .tgz archive
pub fn pack(config: &PackConfig) -> Result<Vec<u8>, PmError> {
    let files = discover_files(&config.root, &config.include, &config.exclude)?;

    let mut buffer = Vec::new();
    {
        let mut archive = TarBuilder::new(GzEncoder::new(&mut buffer, Compression::best()));

        let pkg_json_path = config.root.join("package.json");
        let pkg_json_content = std::fs::read_to_string(&pkg_json_path)
            .map_err(|e| PmError::IoError(format!("Cannot read package.json: {e}")))?;
        let pkg_json: serde_json::Value = serde_json::from_str(&pkg_json_content)
            .map_err(|e| PmError::IoError(format!("Invalid package.json: {e}")))?;

        let mut normalized = pkg_json.as_object().cloned().unwrap_or_default();
        normalized.remove("scripts");
        normalized.remove("devDependencies");
        let normalized_json = serde_json::to_string_pretty(&normalized)
            .map_err(|e| PmError::IoError(e.to_string()))?;

        let mut header = tar::Header::new_gnu();
        header.set_path("package/package.json").map_err(|e| PmError::IoError(e.to_string()))?;
        header.set_size(normalized_json.len() as u64);
        header.set_mode(0o644);
        header.set_mtime(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        );
        header.set_cksum();
        archive.append(&header, normalized_json.as_bytes())
            .map_err(|e| PmError::IoError(e.to_string()))?;

        for file_path in &files {
            let relative = file_path.strip_prefix(&config.root)
                .map_err(|_| PmError::IoError("Path error".into()))?;
            let archive_path = format!("package/{}", relative.display());

            let data = std::fs::read(file_path)
                .map_err(|e| PmError::IoError(format!("Cannot read {}: {e}", file_path.display())))?;

            let mut header = tar::Header::new_gnu();
            header.set_path(&archive_path).map_err(|e| PmError::IoError(e.to_string()))?;
            header.set_size(data.len() as u64);
            header.set_mode(0o644);
            header.set_mtime(
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            );
            header.set_cksum();
            archive.append(&header, data.as_slice())
                .map_err(|e| PmError::IoError(e.to_string()))?;
        }

        archive.finish().map_err(|e| PmError::IoError(e.to_string()))?;
    }

    if config.sign && config.signing_key.is_some() {
        let signature = sign_tarball(&buffer, config.signing_key.as_ref().unwrap())?;
        let mut signed = Vec::new();
        {
            let mut archive = TarBuilder::new(GzEncoder::new(&mut signed, Compression::best()));
            let mut sig_header = tar::Header::new_gnu();
            sig_header.set_path("package/.klyron_signature")
                .map_err(|e| PmError::IoError(e.to_string()))?;
            sig_header.set_size(signature.len() as u64);
            sig_header.set_mode(0o644);
            sig_header.set_mtime(
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            );
            sig_header.set_cksum();
            archive.append(&sig_header, signature.as_slice())
                .map_err(|e| PmError::IoError(e.to_string()))?;
            archive.finish().map_err(|e| PmError::IoError(e.to_string()))?;
        }
        Ok(signed)
    } else {
        Ok(buffer)
    }
}

fn discover_files(root: &Path, include: &[String], exclude: &[String]) -> Result<Vec<PathBuf>, PmError> {
    let mut files = Vec::new();

    if !include.is_empty() {
        for pattern in include {
            let glob_pattern = root.join(pattern);
            for entry in glob::glob(&glob_pattern.to_string_lossy())
                .map_err(|e| PmError::IoError(format!("Glob error: {e}")))? {
                match entry {
                    Ok(path) if path.is_file() => files.push(path),
                    _ => {}
                }
            }
        }
        return Ok(files);
    }

    collect_files_recursive(root, root, exclude, &mut files)?;
    Ok(files)
}

fn collect_files_recursive(
    root: &Path,
    dir: &Path,
    exclude: &[String],
    files: &mut Vec<PathBuf>,
) -> Result<(), PmError> {
    for entry in std::fs::read_dir(dir).map_err(|e| PmError::IoError(e.to_string()))? {
        let entry = entry.map_err(|e| PmError::IoError(e.to_string()))?;
        let path = entry.path();
        let relative = path.strip_prefix(root).unwrap_or(&path);

        if exclude.iter().any(|p| relative.starts_with(p)) {
            continue;
        }

        if path.is_dir() {
            collect_files_recursive(root, &path, exclude, files)?;
        } else if path.is_file() {
            files.push(path);
        }
    }
    Ok(())
}

fn sign_tarball(tarball: &[u8], key: &[u8]) -> Result<Vec<u8>, PmError> {
    use ed25519_dalek::{SigningKey, Signer};

    let key_bytes: [u8; 32] = match key.try_into() {
        Ok(b) => b,
        Err(_) => return Err(PmError::SignatureError("Invalid signing key length".into())),
    };
    let signing_key = SigningKey::from_bytes(&key_bytes);
    let signature = signing_key.sign(tarball);
    Ok(signature.to_bytes().to_vec())
}
