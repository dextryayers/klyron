#!/usr/bin/env node
const { spawnSync } = require("child_process");
const fs = require("fs");
const path = require("path");
const os = require("os");

const MAX_OUTPUT = 1 << 20;

function writeOutput(output) {
  process.stdout.write(JSON.stringify(output) + "\n");
}

function runCmd(cmd, args, input, timeout = 30) {
  const result = spawnSync(cmd, args, {
    input: input,
    timeout: timeout * 1000,
    maxBuffer: MAX_OUTPUT,
    encoding: "utf-8",
    stdio: ["pipe", "pipe", "pipe"],
  });
  const stdout = (result.stdout || "").slice(0, MAX_OUTPUT);
  const stderr = (result.stderr || "").slice(0, MAX_OUTPUT);
  return {
    stdout,
    stderr,
    exit_code: result.status ?? (result.error ? 1 : 0),
    result: result.status === 0 ? "ok" : "failed",
  };
}

function execCode(code, files, filename) {
  if (!code && (!files || files.length === 0)) {
    return { stdout: "", stderr: "No code provided", exit_code: 1, result: "" };
  }
  
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "klyron-js-"));
  try {
    if (files && files.length > 0) {
      for (const f of files) {
        const fp = path.join(tmpDir, f.name);
        fs.mkdirSync(path.dirname(fp), { recursive: true });
        fs.writeFileSync(fp, f.content);
      }
    }
    
    const entry = filename || "main.mjs";
    const fp = path.join(tmpDir, entry);
    fs.mkdirSync(path.dirname(fp), { recursive: true });
    if (code) fs.writeFileSync(fp, code);
    
    const result = runCmd("node", [fp]);
    return result;
  } finally {
    fs.rmSync(tmpDir, { recursive: true, force: true });
  }
}

function evalExpr(expr) {
  try {
    const result = eval(expr);
    return { stdout: String(result ?? ""), stderr: "", exit_code: 0, result: JSON.stringify(result) };
  } catch (e) {
    return { stdout: "", stderr: e.message, exit_code: 1, result: "" };
  }
}

function lintCode(code, filename) {
  if (!code) return { stdout: "", stderr: "No code", exit_code: 1, result: "" };
  const tmpFile = path.join(os.tmpdir(), `klyron-js-lint-${Date.now()}.js`);
  try {
    fs.writeFileSync(tmpFile, code);
    // Try eslint first
    const eslintResult = runCmd("npx", ["--yes", "eslint", "--format", "json", tmpFile]);
    if (eslintResult.exit_code === 0 || eslintResult.stdout) {
      return { stdout: "", stderr: eslintResult.stdout || eslintResult.stderr, exit_code: eslintResult.exit_code, result: eslintResult.stdout };
    }
    // Fallback: try node --check
    const checkResult = runCmd("node", ["--check", tmpFile]);
    return { stdout: "", stderr: checkResult.stderr || "Lint passed", exit_code: checkResult.exit_code, result: checkResult.exit_code === 0 ? "No errors" : checkResult.stderr };
  } finally {
    try { fs.unlinkSync(tmpFile); } catch {}
  }
}

function formatCode(code, filename) {
  if (!code) return { stdout: "", stderr: "No code", exit_code: 1, result: "" };
  // Try prettier
  const result = runCmd("npx", ["--yes", "prettier", "--stdin-filepath", filename || "input.js"], code, 15);
  if (result.exit_code === 0 && result.stdout) {
    return { stdout: result.stdout, stderr: "", exit_code: 0, result: "ok" };
  }
  return { stdout: code, stderr: "Prettier not available", exit_code: 0, result: "ok" };
}

function checkProject(projectDir) {
  if (!fs.existsSync(path.join(projectDir, "package.json"))) {
    return { stdout: "", stderr: "No package.json found", exit_code: 1, result: "" };
  }
  const result = runCmd("npx", ["--yes", "tsc", "--noEmit"], null, 60);
  return result;
}

function buildProject(projectDir) {
  if (!fs.existsSync(path.join(projectDir, "package.json"))) {
    return { stdout: "", stderr: "No package.json found", exit_code: 1, result: "" };
  }
  const result = runCmd("npm", ["run", "build"], null, 120);
  return result;
}

function testProject(projectDir) {
  if (!fs.existsSync(path.join(projectDir, "package.json"))) {
    return { stdout: "", stderr: "No package.json found", exit_code: 1, result: "" };
  }
  const result = runCmd("npm", ["test"], null, 120);
  return result;
}

// Main loop
const readline = require("readline");
const rl = readline.createInterface({ input: process.stdin, output: process.stdout, terminal: false });

rl.on("line", (line) => {
  line = line.trim();
  if (!line) return;
  
  let input;
  try { input = JSON.parse(line); } catch (e) { writeOutput({ stdout: "", stderr: "Invalid JSON: " + e.message, exit_code: 1, result: "" }); return; }
  
  const { action, code, args, filename, files, project } = input;
  let output;
  
  switch (action) {
    case "exec":
    case "run":
      output = execCode(code || "", files || [], filename || "");
      break;
    case "file":
      output = execCode("", files || [], filename || args || "");
      break;
    case "eval":
      output = evalExpr(code || "");
      break;
    case "lint":
      output = lintCode(code || "", filename || "input.js");
      break;
    case "format":
      output = formatCode(code || "", filename || "input.js");
      break;
    case "check":
      output = checkProject(project || ".");
      break;
    case "build":
      output = buildProject(project || ".");
      break;
    case "test":
      output = testProject(project || ".");
      break;
    case "ping":
      output = { stdout: "pong", stderr: "", exit_code: 0, result: "ok" };
      break;
    default:
      output = { stdout: "", stderr: "Unknown action: " + action, exit_code: 1, result: "" };
  }
  
  writeOutput(output);
});
