#!/usr/bin/env node
// Klyron bin entry — spawns the downloaded klyron binary with the same
// arguments passed to this script.

const fs = require("fs");
const path = require("path");
const { spawnSync } = require("child_process");

function findBinary() {
  // 1. Check marker written by install.js
  const marker = path.join(__dirname, ".binary_path");
  if (fs.existsSync(marker)) {
    const p = fs.readFileSync(marker, "utf8").trim();
    if (fs.existsSync(p)) return p;
  }

  // 2. Check bin/ directory
  const binName = process.platform === "win32" ? "klyron.exe" : "klyron";
  const localBin = path.join(__dirname, "bin", binName);
  if (fs.existsSync(localBin)) return localBin;

  // 3. Check PATH
  const which = process.platform === "win32" ? "where" : "which";
  try {
    const result = require("child_process").execSync(`${which} klyron`, {
      encoding: "utf8",
      stdio: ["pipe", "pipe", "ignore"],
    });
    return result.trim().split("\n")[0];
  } catch (_) {}

  // 4. Fallback: same directory as this script
  const fallback = path.join(__dirname, binName);
  if (fs.existsSync(fallback)) return fallback;

  return null;
}

const bin = findBinary();

if (!bin) {
  console.error(
    "[klyron] Binary not found. Please run 'npm install -g klyron' or install manually:\n" +
    "  curl -fsSL https://raw.githubusercontent.com/dextryayers/klyron/main/install.sh | bash"
  );
  process.exit(1);
}

const args = process.argv.slice(2);
const result = spawnSync(bin, args, {
  stdio: "inherit",
  env: process.env,
});

if (result.error) {
  console.error(`[klyron] Failed to execute binary: ${result.error.message}`);
  process.exit(1);
}

process.exit(result.status ?? 0);
