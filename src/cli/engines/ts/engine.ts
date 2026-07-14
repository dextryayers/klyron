function readLine() {
  return new Promise((resolve) => {
    let buffer = "";
    const stdin = process.stdin;
    stdin.setEncoding("utf-8");
    const onData = (chunk) => {
      buffer += chunk;
      const idx = buffer.indexOf("\n");
      if (idx >= 0) {
        const line = buffer.slice(0, idx);
        buffer = buffer.slice(idx + 1);
        stdin.off("data", onData);
        stdin.off("end", onEnd);
        resolve(line);
      }
    };
    const onEnd = () => {
      stdin.off("data", onData);
      stdin.off("end", onEnd);
      resolve(buffer.length > 0 ? buffer : null);
    };
    stdin.on("data", onData);
    stdin.on("end", onEnd);
  });
}

function writeOutput(output) {
  process.stdout.write(JSON.stringify(output) + "\n");
}

function writeError(message, exitCode = 1) {
  writeOutput({
    stdout: "", stderr: message, exit_code: exitCode, result: "",
    diagnostics: [{ file: "", line: 0, col: 0, message, severity: "error" }],
  });
}

const VLQ_CHARS = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

function vlqEncode(value) {
  let result = "";
  let v = value < 0 ? ((-value) << 1) | 1 : value << 1;
  while (v >= 32) {
    result += VLQ_CHARS[(v & 31) | 32];
    v >>>= 5;
  }
  result += VLQ_CHARS[v];
  return result;
}

function generateSourceMap(tsSource, jsSource, filename) {
  const tsLines = tsSource.split("\n");
  const jsLines = jsSource.split("\n");
  const segments = [];
  let srcLine = 0;
  for (let j = 0; j < jsLines.length; j++) {
    if (j === 0 && jsLines[0].includes("use strict")) {
      segments.push(vlqEncode(0));
      continue;
    }
    if (srcLine < tsLines.length) {
      const seg = `${vlqEncode(0)}${vlqEncode(srcLine)}${vlqEncode(0)}${vlqEncode(0)}`;
      segments.push(seg);
      srcLine++;
    } else {
      segments.push(vlqEncode(0));
    }
  }

  const map = {
    version: 3,
    file: filename.replace(/\.(ts|tsx)$/, ".js"),
    sources: [filename],
    sourcesContent: [tsSource],
    names: [],
    mappings: segments.join(";"),
  };
  return Buffer.from(JSON.stringify(map)).toString("base64");
}

function appendSourceMap(js, tsSource, filename) {
  const b64 = generateSourceMap(tsSource, js, filename);
  return `${js}\n//# sourceMappingURL=data:application/json;base64,${b64}`;
}

function parseCompileOptions(args) {
  if (!args) return {};
  const opts = {};
  const parts = args.split(/\s+/);
  for (let i = 0; i < parts.length; i++) {
    const part = parts[i];
    if (!part.startsWith("--")) continue;
    const keyEqValue = part.slice(2);
    const eqIdx = keyEqValue.indexOf("=");
    if (eqIdx >= 0) {
      opts[keyEqValue.slice(0, eqIdx)] = keyEqValue.slice(eqIdx + 1);
    } else {
      const key = keyEqValue;
      if (i + 1 < parts.length && !parts[i + 1].startsWith("--")) {
        opts[key] = parts[i + 1];
        i++;
      } else {
        opts[key] = "true";
      }
    }
  }
  return opts;
}

const STRICT_FLAGS = [
  "strictNullChecks",
  "strictFunctionTypes",
  "strictBindCallApply",
  "strictPropertyInitialization",
  "noImplicitAny",
  "noImplicitThis",
  "alwaysStrict",
  "useUnknownInCatchVariables",
  "noUncheckedIndexedAccess",
];

function loadTsconfigFile(tsconfigPath) {
  const fs = require("fs");
  const content = fs.readFileSync(tsconfigPath, "utf-8");
  return JSON.parse(content);
}

function buildCompilerOptions(options, isJSX) {
  const opts = {};

  opts.target = options.target || "ES2022";
  opts.module = options.module || "ESNext";
  opts.skipLibCheck = true;
  opts.forceConsistentCasingInFileNames = true;

  if (options.strict === "false") {
    opts.strict = false;
  } else {
    opts.strict = true;
  }
  for (const flag of STRICT_FLAGS) {
    if (flag in options) {
      opts[flag] = options[flag] !== "false";
    }
  }

  if (isJSX) {
    opts.jsx = options.jsx || "react-jsx";
  }
  if (options.experimentalDecorators === "true") {
    opts.experimentalDecorators = true;
  }
  if (options.emitDecoratorMetadata === "true") {
    opts.emitDecoratorMetadata = true;
  }
  if (options.moduleResolution) {
    opts.moduleResolution = options.moduleResolution;
  }
  if (options.declaration === "true") opts.declaration = true;
  if (options.declarationMap === "true") opts.declarationMap = true;
  if (options.emitDeclarationOnly === "true") opts.emitDeclarationOnly = true;
  if (options.composite === "true") opts.composite = true;
  if (options.tsBuildInfoFile) opts.tsBuildInfoFile = options.tsBuildInfoFile;
  if (options.paths) {
    try { opts.paths = JSON.parse(options.paths); } catch { opts.paths = {}; }
  }
  if (options.incremental === "true") opts.incremental = true;
  if (options.isolatedModules === "true") opts.isolatedModules = true;
  if (options.outDir) opts.outDir = options.outDir;
  if (options.rootDir) opts.rootDir = options.rootDir;

  return opts;
}

function buildTsconfigConfig(options, isJSX, includePatterns, references) {
  const compilerOptions = buildCompilerOptions(options, isJSX);
  const config = { compilerOptions };

  if (includePatterns && includePatterns.length > 0) {
    config.include = includePatterns;
  }
  if (references && references.length > 0) {
    config.references = references.map((r) => ({ path: r.trim() }));
  } else if (options.references) {
    config.references = options.references.split(",").map((r) => ({ path: r.trim() }));
  }

  return config;
}

function stripAnsi(str) {
  return str.replace(/\u001b\[[0-9;]*m/g, "");
}

function transpileTypeScript(code, filename = "input.ts", options = {}) {
  const isJSX = filename.endsWith(".tsx") || filename.endsWith(".jsx");

  try {
    if (typeof Deno !== "undefined" && Deno.transpileOnly) {
      const result = Deno.transpileOnly({ [filename]: code }, {});
      const js = result?.[filename]?.source;
      if (js) return { js };
    }
  } catch { /* ignore */ }

  try {
    if (typeof require !== "undefined") {
      const esbuild = require("esbuild");
      const tsconfigRaw = {};
      const ro = {};
      if (options.experimentalDecorators === "true") {
        ro.experimentalDecorators = true;
      }
      if (options.emitDecoratorMetadata === "true") {
        ro.emitDecoratorMetadata = true;
      }
      if (Object.keys(ro).length > 0) {
        tsconfigRaw.compilerOptions = ro;
      }
      const result = esbuild.transformSync(code, {
        loader: isJSX ? "tsx" : "ts",
        target: options.target || "es2022",
        sourcemap: "inline",
        jsx: isJSX ? "automatic" : undefined,
        tsconfigRaw: Object.keys(tsconfigRaw).length > 0 ? tsconfigRaw : undefined,
      });
      if (result?.code) return { js: result.code, sourceMap: result.map };
    }
  } catch { /* ignore */ }

  const js = stripTypeAnnotations(code, isJSX);
  return { js };
}

const TS_PATTERNS = [
  [/:?\s*(?:number|string|boolean|void|null|undefined|any|never|unknown|bigint|symbol|object)\s*(?=[=,);\]}:]|$)/g, ""],
  [/:?\s*[A-Z][A-Za-z_$<>[\]\s,|&]*(?:\s*[|&]\s*[A-Z][A-Za-z_$<>[\]\s,|&]*)*\s*(?=[=,);\]}:]|$)/g, ""],
  [/\s+as\s+(?:const|[A-Za-z_$][\w$<>[\]\s,|&]*)/g, ""],
  [/\s+satisfies\s+[A-Za-z_$][\w$<>[\]\s,|&]*/g, ""],
  [/interface\s+\w+(?:\s*extends\s+[^{]+)?\s*\{[^}]*\}\s*/g, ""],
  [/type\s+\w+(?:<[^>]+>)?\s*=\s*[^;]+;/g, ""],
  [/\benum\s+(\w+)\s*\{([^}]*)\}/g, (_m, name, body) => {
    const entries = body.split(",").map((e) => {
      const parts = e.trim().split(/\s*=\s*/);
      if (parts.length >= 2) return `${parts[0].trim()}: ${parts[1].trim()}`;
      return `${parts[0].trim()}: ${JSON.stringify(parts[0].trim())}`;
    });
    return `const ${name} = Object.freeze({ ${entries.join(", ")} });`;
  }],
  [/\bdeclare\s+/g, ""],
  [/\b(?:public|private|protected|readonly|abstract)\s+/g, ""],
  [/!\s*(?=[=:;])/g, ""],
  [/(\w+)\?\s*(?=[:;])/g, "$1"],
  [/@\w+(?:\.\w+)*(?:\([^)]*\))?\s*/g, ""],
  [/import\s+type\s+\{[^}]*\}\s*from\s*['"][^'"]+['"];\s*/g, ""],
  [/\bexport\s+type\s+/g, "export "],
  [/\bexport\s+interface\s+/g, "export "],
  [/\bas\s+const\b/g, ""],
];

function stripTypeAnnotations(code, isJSX = false) {
  let result = code;
  const placeholders = [];
  let idx = 0;

  if (isJSX) {
    result = result.replace(
      /<[A-Za-z_$][\w$]*(?:\s+[^>]*\/\s*>|\s+[^>]*>[\s\S]*?<\/[A-Za-z_$][\w$]*>|\s*\/>|>[\s\S]*?<\/[A-Za-z_$][\w$]*>)/g,
      (m) => {
        const ph = `__PH_${idx}__`;
        placeholders[idx++] = m;
        return ph;
      }
    );
  }

  result = result.replace(/"[^"\\]*(?:\\.[^"\\]*)*"|'[^'\\]*(?:\\.[^'\\]*)*'|`[^`\\]*(?:\\.[^`\\]*)*`/g, (m) => {
    const ph = `__PH_${idx}__`;
    placeholders[idx++] = m;
    return ph;
  });

  for (const [pattern, replacement] of TS_PATTERNS) {
    result = result.replace(pattern, replacement);
  }

  result = result.replace(/^\/\/\/\s*<reference\s+path\s*=.*\/>$/gm, "");
  result = result.replace(/^\/\/\/\s*<reference\s+types\s*=.*\/>$/gm, "");
  result = result.replace(/^import\s+['"][^'"]+['"];\s*$/gm, "");

  for (let i = 0; i < placeholders.length; i++) {
    result = result.replace(`__PH_${i}__`, placeholders[i]);
  }

  return result;
}

function parseTscDiagnostics(stdout, stderr, filename) {
  const diags = [];
  const cleanStdout = stripAnsi(stdout);
  const cleanStderr = stripAnsi(stderr);
  const allOutput = cleanStdout + "\n" + cleanStderr;

  const lineRegex = /^(.+?)\((\d+),(\d+)\):\s+(error|warning)\s+(TS\d+):\s+(.+)$/gm;
  let m;
  while ((m = lineRegex.exec(allOutput)) !== null) {
    diags.push({
      file: m[1],
      line: parseInt(m[2]),
      col: parseInt(m[3]),
      message: `${m[5]}: ${m[6]}`,
      severity: m[4],
    });
  }

  if (diags.length === 0) {
    const prettyRegex = /^(.+?):(\d+):(\d+)\s+-\s+(error|warning)\s+(TS\d+):\s+(.+)$/gm;
    while ((m = prettyRegex.exec(allOutput)) !== null) {
      diags.push({
        file: m[1],
        line: parseInt(m[2]),
        col: parseInt(m[3]),
        message: `${m[5]}: ${m[6]}`,
        severity: m[4],
      });
    }
  }

  if (diags.length === 0) {
    const simpleRegex = /^(.+?)\((\d+),(\d+)\):\s+(.+)$/gm;
    while ((m = simpleRegex.exec(allOutput)) !== null) {
      diags.push({
        file: m[1],
        line: parseInt(m[2]),
        col: parseInt(m[3]),
        message: m[4],
        severity: "error",
      });
    }
  }

  if (diags.length === 0 && (stdout + stderr).trim()) {
    const output = (stdout + stderr).trim();
    if (output.toLowerCase().includes("error")) {
      diags.push({
        file: filename,
        line: 0,
        col: 0,
        message: output.split("\n").slice(0, 3).join("\n"),
        severity: "error",
      });
    }
  }

  return diags;
}

async function runTsc(code, filename, options = {}, extraFiles = []) {
  const fs = await import("fs");
  const os = await import("os");
  const path = await import("path");
  const cp = require("child_process");

  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "klyron-tsc-"));
  const isJSX = filename.endsWith(".tsx") || filename.endsWith(".jsx");

  const allFiles = [];
  allFiles.push({ name: filename || "input.ts", content: code });
  for (const f of extraFiles) {
    if (f.name === filename) continue;
    allFiles.push(f);
  }

  for (const f of allFiles) {
    const fp = path.join(tmpDir, f.name);
    const dir = path.dirname(fp);
    if (!fs.existsSync(dir)) fs.mkdirSync(dir, { recursive: true });
    fs.writeFileSync(fp, f.content, "utf-8");
  }

  const includePatterns = allFiles.map((f) => f.name);
  let config;

  if (options.tsconfig) {
    try {
      config = loadTsconfigFile(options.tsconfig);
      const cliOpts = buildCompilerOptions(options, isJSX);
      if (!config.compilerOptions) config.compilerOptions = {};
      Object.assign(config.compilerOptions, cliOpts);
      config.include = includePatterns;
    } catch (e) {
      try { fs.rmSync(tmpDir, { recursive: true, force: true }); } catch {}
      throw new Error(`Failed to load tsconfig '${options.tsconfig}': ${e.message}`);
    }
  } else {
    config = buildTsconfigConfig(options, isJSX, includePatterns);
  }

  const configFile = path.join(tmpDir, "tsconfig.json");
  fs.writeFileSync(configFile, JSON.stringify(config, null, 2), "utf-8");

  const diags = [];
  let tscStdout = "";
  let tscStderr = "";

  try {
    const pretty = options.pretty === "true";
    const tscExtraArgs = [];

    if (options.noEmit !== "false") {
      tscExtraArgs.push("--noEmit");
    }
    if (pretty) {
      tscExtraArgs.push("--pretty");
    } else {
      tscExtraArgs.push("--pretty", "false");
    }

    const tscCmds = [
      ["npx", "--yes", "typescript", "--project", configFile, ...tscExtraArgs],
      ["tsc", "--project", configFile, ...tscExtraArgs],
    ];

    let stdout = "";
    let stderr = "";
    for (const cmd of tscCmds) {
      try {
        const result = cp.spawnSync(cmd[0], cmd.slice(1), {
          cwd: tmpDir,
          timeout: 30000,
          encoding: "utf-8",
          stdio: ["ignore", "pipe", "pipe"],
        });
        stdout = result.stdout || "";
        stderr = result.stderr || "";
        break;
      } catch { continue; }
    }

    tscStdout = stdout;
    tscStderr = stderr;
    diags.push(...parseTscDiagnostics(stdout, stderr, filename));
  } catch (e) {
    if (diags.length === 0) {
      diags.push({ file: filename, line: 0, col: 0, message: `tsc check failed: ${e.message}`, severity: "error" });
    }
  } finally {
    try { fs.rmSync(tmpDir, { recursive: true, force: true }); } catch { /* ignore */ }
  }

  return { diags, stdout: tscStdout, stderr: tscStderr };
}

async function compileCode(code, filename, options = {}, files = []) {
  const useTsc = !!options.outDir || files.length > 0 ||
    options.declaration === "true" || options.declarationMap === "true" ||
    options.emitDeclarationOnly === "true";

  if (useTsc) {
    return compileWithTsc(code, filename, options, files);
  }

  try {
    const { js } = transpileTypeScript(code, filename, options);
    return {
      stdout: js,
      stderr: "",
      exit_code: 0,
      result: js,
    };
  } catch (e) {
    const err = e instanceof Error ? e : new Error(String(e));
    return {
      stdout: "",
      stderr: err.message,
      exit_code: 1,
      result: "",
      diagnostics: extractDiagnostics(err, code),
    };
  }
}

async function compileWithTsc(code, filename, options = {}, extraFiles = []) {
  const fs = await import("fs");
  const os = await import("os");
  const path = await import("path");
  const cp = require("child_process");

  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "klyron-tsc-compile-"));
  const isJSX = filename.endsWith(".tsx") || filename.endsWith(".jsx");

  const allFiles = [];
  allFiles.push({ name: filename || "input.ts", content: code });
  for (const f of extraFiles) {
    if (f.name === filename) continue;
    allFiles.push(f);
  }

  for (const f of allFiles) {
    const fp = path.join(tmpDir, f.name);
    const dir = path.dirname(fp);
    if (!fs.existsSync(dir)) fs.mkdirSync(dir, { recursive: true });
    fs.writeFileSync(fp, f.content, "utf-8");
  }

  const resolvedOutDir = options.outDir
    ? path.resolve(options.outDir)
    : path.join(tmpDir, "out");

  if (!fs.existsSync(resolvedOutDir)) {
    fs.mkdirSync(resolvedOutDir, { recursive: true });
  }

  const compileOptions = { ...options, outDir: resolvedOutDir };
  if (compileOptions.noEmit !== "true") {
    compileOptions.noEmit = "false";
  }

  const includePatterns = allFiles.map((f) => f.name);
  let config;

  if (compileOptions.tsconfig) {
    try {
      config = loadTsconfigFile(compileOptions.tsconfig);
      const cliOpts = buildCompilerOptions(compileOptions, isJSX);
      if (!config.compilerOptions) config.compilerOptions = {};
      Object.assign(config.compilerOptions, cliOpts);
      config.include = includePatterns;
    } catch (e) {
      try { fs.rmSync(tmpDir, { recursive: true, force: true }); } catch {}
      return {
        stdout: "",
        stderr: `Failed to load tsconfig '${compileOptions.tsconfig}': ${e.message}`,
        exit_code: 1,
        result: "",
        diagnostics: [{ file: filename, line: 0, col: 0, message: e.message, severity: "error" }],
      };
    }
  } else {
    config = buildTsconfigConfig(compileOptions, isJSX, includePatterns);
  }

  const configFile = path.join(tmpDir, "tsconfig.json");
  fs.writeFileSync(configFile, JSON.stringify(config, null, 2), "utf-8");

  try {
    const pretty = options.pretty === "true";
    const tscArgs = ["--project", configFile];
    if (pretty) {
      tscArgs.push("--pretty");
    } else {
      tscArgs.push("--pretty", "false");
    }

    const tscCmds = [
      ["npx", "--yes", "typescript", ...tscArgs],
      ["tsc", ...tscArgs],
    ];

    let stdout = "";
    let stderr = "";
    for (const cmd of tscCmds) {
      try {
        const result = cp.spawnSync(cmd[0], cmd.slice(1), {
          cwd: tmpDir,
          timeout: 60000,
          encoding: "utf-8",
          stdio: ["ignore", "pipe", "pipe"],
        });
        stdout = result.stdout || "";
        stderr = result.stderr || "";
        break;
      } catch { continue; }
    }

    const diags = parseTscDiagnostics(stdout, stderr, filename);
    const hasErrors = diags.some((d) => d.severity === "error");

    if (hasErrors && compileOptions.noEmitOnError === "true") {
      try { fs.rmSync(tmpDir, { recursive: true, force: true }); } catch {}
      return {
        stdout,
        stderr,
        exit_code: 1,
        result: "Compilation failed with type errors (noEmitOnError)",
        diagnostics: diags,
      };
    }

    const emittedFiles = [];
    if (fs.existsSync(resolvedOutDir)) {
      const collectFiles = (dir, prefix = "") => {
        for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
          const full = path.join(dir, entry.name);
          if (entry.isDirectory()) {
            collectFiles(full, prefix + entry.name + "/");
          } else {
            emittedFiles.push(prefix + entry.name);
          }
        }
      };
      collectFiles(resolvedOutDir);
    }

    let mainJs = "";
    const mainOutName = filename.replace(/\.(ts|tsx)$/, ".js");
    const mainOutPath = path.join(resolvedOutDir, mainOutName);
    if (fs.existsSync(mainOutPath)) {
      mainJs = fs.readFileSync(mainOutPath, "utf-8");
    } else if (emittedFiles.length > 0) {
      for (const ef of emittedFiles) {
        const efPath = path.join(resolvedOutDir, ef);
        if (ef.endsWith(".js") && !ef.endsWith(".d.ts") && fs.existsSync(efPath)) {
          mainJs = fs.readFileSync(efPath, "utf-8");
          break;
        }
      }
    }

    try { fs.rmSync(tmpDir, { recursive: true, force: true }); } catch {}

    return {
      stdout: mainJs,
      stderr: emittedFiles.length > 0 ? `Emitted: ${emittedFiles.join(", ")}` : "",
      exit_code: hasErrors ? 1 : 0,
      result: mainJs,
      diagnostics: diags,
    };
  } catch (e) {
    try { fs.rmSync(tmpDir, { recursive: true, force: true }); } catch {}
    return {
      stdout: "",
      stderr: e.message,
      exit_code: 1,
      result: "",
      diagnostics: [{ file: filename, line: 0, col: 0, message: e.message, severity: "error" }],
    };
  }
}

async function watchCode(code, filename, files = [], options = {}) {
  const fs = await import("fs");
  const os = await import("os");
  const path = await import("path");
  const cp = require("child_process");

  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "klyron-tsw-"));
  const isJSX = filename.endsWith(".tsx") || filename.endsWith(".jsx");

  const srcFile = path.join(tmpDir, filename || "input.ts");
  fs.writeFileSync(srcFile, code, "utf-8");
  for (const f of files) {
    const fp = path.join(tmpDir, f.name);
    const dir = path.dirname(fp);
    if (!fs.existsSync(dir)) fs.mkdirSync(dir, { recursive: true });
    fs.writeFileSync(fp, f.content, "utf-8");
  }

  const includePatterns = [filename || "input.ts"];
  for (const f of files) {
    if (!includePatterns.includes(f.name)) includePatterns.push(f.name);
  }

  let config;
  if (options.tsconfig) {
    try {
      config = loadTsconfigFile(options.tsconfig);
      const cliOpts = buildCompilerOptions(options, isJSX);
      if (!config.compilerOptions) config.compilerOptions = {};
      Object.assign(config.compilerOptions, cliOpts);
      config.include = [...(config.include || []), ...includePatterns];
    } catch (e) {
      try { fs.rmSync(tmpDir, { recursive: true, force: true }); } catch {}
      return {
        stdout: "",
        stderr: `Failed to load tsconfig '${options.tsconfig}': ${e.message}`,
        exit_code: 1, result: "",
        diagnostics: [{ file: filename, line: 0, col: 0, message: e.message, severity: "error" }],
      };
    }
  } else {
    config = buildTsconfigConfig(options, isJSX, includePatterns);
  }

  fs.writeFileSync(path.join(tmpDir, "tsconfig.json"), JSON.stringify(config, null, 2), "utf-8");

  return new Promise((resolve) => {
    let resolved = false;
    let stdout = "";
    let stderr = "";

    const pretty = options.pretty === "true";
    const tscExtraArgs = [];
    tscExtraArgs.push("--project", tmpDir);
    tscExtraArgs.push("--noEmit");
    if (pretty) {
      tscExtraArgs.push("--pretty");
    } else {
      tscExtraArgs.push("--pretty", "false");
    }
    tscExtraArgs.push("--watch", "--preserveWatchOutput");

    const child = cp.spawn("npx", ["--yes", "typescript", ...tscExtraArgs], {
      cwd: tmpDir,
      encoding: "utf-8",
      stdio: ["ignore", "pipe", "pipe"],
    });

    const timer = setTimeout(() => {
      if (!resolved) {
        resolved = true;
        child.kill();
        const diags = parseTscDiagnostics(stdout, stderr, filename);
        cleanup();
        resolve({
          stdout, stderr,
          exit_code: diags.filter(d => d.severity === "error").length > 0 ? 1 : 0,
          result: diags.filter(d => d.severity === "error").length > 0 ? "Type errors found" : "No type errors",
          diagnostics: diags,
        });
      }
    }, 5000);

    child.stdout?.on("data", (data) => {
      stdout += data;
      if (!resolved && (data.includes("Found 0 errors") || data.includes(" error") || data.includes("watch")) && stdout.includes("Found")) {
        clearTimeout(timer);
        resolved = true;
        child.kill();
        const diags = parseTscDiagnostics(stdout, stderr, filename);
        cleanup();
        resolve({
          stdout, stderr,
          exit_code: diags.filter(d => d.severity === "error").length > 0 ? 1 : 0,
          result: diags.filter(d => d.severity === "error").length > 0 ? "Type errors found" : "No type errors",
          diagnostics: diags,
        });
      }
    });

    child.stderr?.on("data", (data) => {
      stderr += data;
    });

    child.on("error", (err) => {
      if (!resolved) {
        clearTimeout(timer);
        resolved = true;
        cleanup();
        resolve({
          stdout, stderr: `Watch failed: ${err.message}`,
          exit_code: 1, result: "",
          diagnostics: [{ file: filename, line: 0, col: 0, message: err.message, severity: "error" }],
        });
      }
    });

    child.on("close", () => {
      if (!resolved) {
        clearTimeout(timer);
        resolved = true;
        const diags = parseTscDiagnostics(stdout, stderr, filename);
        cleanup();
        resolve({
          stdout, stderr,
          exit_code: diags.filter(d => d.severity === "error").length > 0 ? 1 : 0,
          result: diags.filter(d => d.severity === "error").length > 0 ? "Type errors found" : "No type errors",
          diagnostics: diags,
        });
      }
    });

    function cleanup() {
      try { fs.rmSync(tmpDir, { recursive: true, force: true }); } catch { /* ignore */ }
    }
  });
}

function extractDiagnostics(err, sourceTs) {
  const msg = err.message;
  const lines = msg.split("\n");
  const diags = [];
  const stackLineRegex = /at\s+(?:.*?\s+)?\(?(.+?):(\d+):(\d+)\)?$/;

  for (const line of lines) {
    const m = line.match(/(.+?)\((\d+),(\d+)\):\s+(.+)/);
    if (m) {
      let jsLine = parseInt(m[2]) - 1;
      let tsLine = jsLine;
      if (sourceTs) {
        tsLine = Math.min(jsLine, sourceTs.split("\n").length - 1);
      }
      diags.push({ file: m[1], line: tsLine + 1, col: parseInt(m[3]), message: m[4], severity: "error" });
      continue;
    }
    const sm = line.match(stackLineRegex);
    if (sm) {
      diags.push({ file: sm[1], line: parseInt(sm[2]), col: parseInt(sm[3]), message: "Runtime error", severity: "error" });
    }
  }

  if (diags.length === 0) {
    diags.push({ file: "<eval>", line: 0, col: 0, message: msg, severity: "error" });
  }
  return diags;
}

async function execCode(code, filename = "input.ts", options = {}) {
  try {
    const { js } = transpileTypeScript(code, filename, options);
    const jsWithMap = appendSourceMap(js, code, filename);
    const origConsole = globalThis.console;
    const capturedOutput = [];
    globalThis.console = {
      log: (...args) => capturedOutput.push(args.map((a) => typeof a === "object" ? JSON.stringify(a, null, 2) : String(a)).join(" ") + "\n"),
      warn: (...args) => capturedOutput.push(args.map(String).join(" ") + "\n"),
      error: (...args) => capturedOutput.push(args.map(String).join(" ") + "\n"),
      info: (...args) => capturedOutput.push(args.map(String).join(" ") + "\n"),
    };
    try {
      const asyncFn = new Function(`"use strict"; return (async () => {\n${jsWithMap}\n})();`);
      const result = await asyncFn();
      return { stdout: capturedOutput.join(""), stderr: "", exit_code: 0, result: String(result ?? "") };
    } finally {
      globalThis.console = origConsole;
    }
  } catch (e) {
    const err = e instanceof Error ? e : new Error(String(e));
    return {
      stdout: "", stderr: err.message, exit_code: 1, result: "",
      diagnostics: extractDiagnostics(err, code),
    };
  }
}

async function execFile(filename) {
  try {
    const fs = await import("fs");
    const path = await import("path");
    if (!fs.existsSync(filename)) {
      return { stdout: "", stderr: `File not found: ${filename}`, exit_code: 1, result: "" };
    }
    const code = fs.readFileSync(filename, "utf-8");
    const baseName = path.basename(filename);
    const { js } = transpileTypeScript(code, baseName);
    const jsWithMap = appendSourceMap(js, code, baseName);
    const origConsole = globalThis.console;
    const capturedOutput = [];
    globalThis.console = {
      log: (...args) => capturedOutput.push(args.map((a) => typeof a === "object" ? JSON.stringify(a, null, 2) : String(a)).join(" ") + "\n"),
      warn: (...args) => capturedOutput.push(args.map(String).join(" ") + "\n"),
      error: (...args) => capturedOutput.push(args.map(String).join(" ") + "\n"),
      info: (...args) => capturedOutput.push(args.map(String).join(" ") + "\n"),
    };
    try {
      const asyncFn = new Function(`"use strict"; return (async () => {\n${jsWithMap}\n})();`);
      const result = await asyncFn();
      return { stdout: capturedOutput.join(""), stderr: "", exit_code: 0, result: String(result ?? "") };
    } finally {
      globalThis.console = origConsole;
    }
  } catch (e) {
    const err = e instanceof Error ? e : new Error(String(e));
    return {
      stdout: "", stderr: err.message, exit_code: 1, result: "",
      diagnostics: extractDiagnostics(err),
    };
  }
}

async function evalExpr(expr) {
  try {
    const origConsole = globalThis.console;
    const capturedOutput = [];
    globalThis.console = {
      log: (...args) => capturedOutput.push(args.map((a) => typeof a === "object" ? JSON.stringify(a, null, 2) : String(a)).join(" ") + "\n"),
      warn: (...args) => capturedOutput.push(args.map(String).join(" ") + "\n"),
      error: (...args) => capturedOutput.push(args.map(String).join(" ") + "\n"),
      info: (...args) => capturedOutput.push(args.map(String).join(" ") + "\n"),
    };
    try {
      const js = stripTypeAnnotations(expr);
      const result = eval(js);
      return { stdout: capturedOutput.join("") + String(result ?? ""), stderr: "", exit_code: 0, result: JSON.stringify(result) };
    } finally {
      globalThis.console = origConsole;
    }
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    return { stdout: "", stderr: msg, exit_code: 1, result: "" };
  }
}

async function checkType(code, filename = "input.ts", options = {}, files = []) {
  try {
    const { diags, stdout, stderr } = await runTsc(code, filename, options, files);
    const errors = diags.filter(d => d.severity === "error");
    return {
      stdout,
      stderr,
      exit_code: errors.length > 0 ? 1 : 0,
      result: errors.length > 0 ? "Type errors found" : "No type errors",
      diagnostics: diags,
    };
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    return { stdout: "", stderr: msg, exit_code: 1, result: "", diagnostics: [] };
  }
}

async function formatCode(code: string, filename: string): Promise<EngineOutput> {
  try {
    const fs = await import("fs");
    const path = await import("path");
    // Try prettier first
    try {
      const result = cp.spawnSync("npx", ["--yes", "prettier", "--parser", "typescript", "--stdin-filepath", filename], {
        input: code,
        timeout: 30000,
        encoding: "utf-8",
        stdio: ["pipe", "pipe", "pipe"],
      });
      if (result.status === 0 && result.stdout) {
        return { stdout: result.stdout, stderr: "", exit_code: 0, result: "ok" };
      }
    } catch {}
    // Fallback: return code as-is with a message
    return { stdout: code, stderr: "Prettier not available, returned original code", exit_code: 0, result: "ok" };
  } catch (e) {
    const err = e instanceof Error ? e : new Error(String(e));
    return { stdout: "", stderr: err.message, exit_code: 1, result: "" };
  }
}

async function lintCode(code: string, filename: string): Promise<EngineOutput> {
  try {
    try {
      const result = cp.spawnSync("npx", ["--yes", "eslint", "--stdin", "--stdin-filepath", filename, "--format", "json"], {
        input: code,
        timeout: 30000,
        encoding: "utf-8",
        stdio: ["pipe", "pipe", "pipe"],
      });
      const lintOutput = result.stdout || result.stderr || "";
      return { stdout: "", stderr: lintOutput, exit_code: result.status ?? 1, result: lintOutput };
    } catch {
      return { stdout: "", stderr: "ESLint not available", exit_code: 0, result: "" };
    }
  } catch (e) {
    const err = e instanceof Error ? e : new Error(String(e));
    return { stdout: "", stderr: err.message, exit_code: 1, result: "" };
  }
}

async function main() {
  process.stdin.setEncoding("utf-8");
  process.on("uncaughtException", (err) => writeError(String(err)));
  process.on("unhandledRejection", (reason) => writeError(String(reason)));

  while (true) {
    const line = await readLine();
    if (line === null) break;

    let input;
    try { input = JSON.parse(line); }
    catch { writeError("Invalid JSON input"); continue; }

    const { action, code, args, filename, files } = input;
    const options = parseCompileOptions(args);
    let output;

    switch (action) {
      case "exec":
        output = await execCode(code ?? "", filename ?? "input.ts", options);
        break;
      case "file":
        output = await execFile(filename ?? args ?? "input.ts");
        break;
      case "eval":
        output = await evalExpr(code ?? "");
        break;
      case "check":
      case "typecheck":
        output = await checkType(code ?? "", filename ?? "input.ts", options, files ?? []);
        break;
      case "compile":
      case "transpile":
        output = await compileCode(code ?? "", filename ?? "input.ts", options, files ?? []);
        break;
      case "watch":
        output = await watchCode(code ?? "", filename ?? "input.ts", files ?? [], options);
        break;
      case "ping":
      case "":
        output = { stdout: "pong", stderr: "", exit_code: 0, result: "ok" };
        break;
      case "format":
        output = await formatCode(code ?? "", filename ?? "input.ts");
        break;
      case "lint":
        output = await lintCode(code ?? "", filename ?? "input.ts");
        break;
      default:
        writeError(`Unknown action: ${action}`);
        continue;
    }
    writeOutput(output);
  }
}

main().catch((e) => writeError(String(e)));
