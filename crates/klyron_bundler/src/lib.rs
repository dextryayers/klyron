use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BundlerKind {
    Esbuild,
    Vite,
    Webpack,
    Rollup,
    Parcel,
    Turbopack,
    Rsbuild,
    SWC,
}

pub struct BundleOptions {
    pub entry: String,
    pub out_dir: String,
    pub minify: bool,
    pub sourcemap: bool,
    pub format: OutputFormat,
}

pub enum OutputFormat {
    Esm,
    Cjs,
    Iife,
}

pub fn detect_bundler(dir: &Path) -> Option<BundlerKind> {
    let configs = [
        ("vite.config.*", BundlerKind::Vite),
        ("next.config.*", BundlerKind::Turbopack),
        ("esbuild.*", BundlerKind::Esbuild),
        ("webpack.config.*", BundlerKind::Webpack),
        ("rollup.config.*", BundlerKind::Rollup),
        ("parcelrc", BundlerKind::Parcel),
        (".parcelrc", BundlerKind::Parcel),
    ];
    for (pattern, kind) in &configs {
        let pat = pattern.replace('*', "");
        for entry in std::fs::read_dir(dir).ok()? {
            if let Ok(e) = entry {
                let name = e.file_name().to_string_lossy().to_string();
                if name.starts_with(&pat.trim_end_matches('.')) && name.contains(&pat.trim_end_matches('*')) {
                    return Some(*kind);
                }
            }
        }
    }
    None
}

pub fn bundle_cmd(kind: BundlerKind) -> &'static str {
    match kind {
        BundlerKind::Esbuild => "esbuild --bundle",
        BundlerKind::Vite => "vite build",
        BundlerKind::Webpack => "webpack --mode production",
        BundlerKind::Rollup => "rollup -c",
        BundlerKind::Parcel => "parcel build",
        BundlerKind::Turbopack => "next build",
        BundlerKind::Rsbuild => "rsbuild build",
        BundlerKind::SWC => "swc",
    }
}
