#!/usr/bin/env node
// Klyron npm postinstall — downloads the correct prebuilt binary for the
// current platform from GitHub Releases.

const fs = require("fs");
const path = require("path");
const https = require("https");
const { execSync } = require("child_process");

const PLATFORMS = require("./platforms.json");
const PKG = require("./package.json");

const REPO = "dextryayers/klyron";

// ── Helpers ─────────────────────────────────────────────────────

function platformKey() {
  const p = process.platform;   // "win32", "darwin", "linux"
  const a = process.arch;       // "x64", "arm64"
  const archMap = { x64: "x86_64", arm64: "aarch64" };
  const mapped = archMap[a] || a;
  return `${p}-${mapped}`;
}

function download(url, dest) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(dest);
    https
      .get(url, (res) => {
        if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
          file.close();
          fs.unlinkSync(dest);
          return download(res.headers.location, dest).then(resolve).catch(reject);
        }
        if (res.statusCode !== 200) {
          file.close();
          fs.unlinkSync(dest);
          return reject(new Error(`Download failed (HTTP ${res.statusCode}): ${url}`));
        }
        res.pipe(file);
        file.on("finish", () => {
          file.close();
          resolve();
        });
      })
      .on("error", (err) => {
        file.close();
        if (fs.existsSync(dest)) fs.unlinkSync(dest);
        reject(err);
      });
  });
}

function extractTarGz(src, destDir) {
  execSync(`tar xzf "${src}" -C "${destDir}"`, { stdio: "pipe" });
}

function extractZip(src, destDir) {
  try {
    execSync(`unzip -o "${src}" -d "${destDir}"`, { stdio: "pipe" });
  } catch (_) {
    execSync(`powershell -Command "Expand-Archive -Path '${src}' -DestinationPath '${destDir}' -Force"`, {
      stdio: "pipe",
    });
  }
}

// ── Main ────────────────────────────────────────────────────────

async function main() {
  const key = platformKey();
  const asset = PLATFORMS[key];

  if (!asset) {
    console.error(`[klyron] Unsupported platform: ${key}`);
    console.error(`[klyron] Supported platforms: ${Object.keys(PLATFORMS).join(", ")}`);
    process.exit(1);
  }

  const version = PKG.version;
  const url = `https://github.com/${REPO}/releases/download/v${version}/${asset}`;
  const downloadDir = path.join(__dirname, ".download");
  const destFile = path.join(downloadDir, asset);
  const binDir = path.join(__dirname, "bin");

  fs.mkdirSync(downloadDir, { recursive: true });
  fs.mkdirSync(binDir, { recursive: true });

  console.log(`[klyron] Downloading ${asset} ...`);

  try {
    await download(url, destFile);
  } catch (err) {
    // Fallback: try latest
    const latestUrl = `https://github.com/${REPO}/releases/latest/download/${asset}`;
    console.log(`[klyron] Version v${version} not found, trying latest ...`);
    try {
      await download(latestUrl, destFile);
    } catch (err2) {
      console.error(`[klyron] Download failed: ${err2.message}`);
      process.exit(1);
    }
  }

  console.log(`[klyron] Extracting ...`);

  if (asset.endsWith(".tar.gz")) {
    extractTarGz(destFile, binDir);
  } else {
    extractZip(destFile, binDir);
  }

  // The binary is at bin/klyron (or bin/klyron.exe on Windows)
  const binName = process.platform === "win32" ? "klyron.exe" : "klyron";
  const binPath = path.join(binDir, binName);

  if (!fs.existsSync(binPath)) {
    console.error(`[klyron] Binary not found after extraction: ${binPath}`);
    process.exit(1);
  }

  // Make executable (Unix)
  if (process.platform !== "win32") {
    fs.chmodSync(binPath, 0o755);
  }

  // Write a marker so klyron.js knows where the binary is
  fs.writeFileSync(path.join(__dirname, ".binary_path"), binPath);

  // Cleanup
  fs.rmSync(downloadDir, { recursive: true, force: true });

  console.log(`[klyron] Installed to ${binPath}`);
}

main().catch((err) => {
  console.error(`[klyron] Installation failed: ${err.message}`);
  process.exit(1);
});
