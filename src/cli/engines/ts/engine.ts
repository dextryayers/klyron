interface FileEntry {
  name: string;
  content: string;
}

interface EngineInput {
  action: string;
  code?: string;
  args?: string;
  filename?: string;
  project?: string;
  files?: FileEntry[];
}

interface Diagnostic {
  file: string; line: number; col: number; message: string; severity: string;
}

interface EngineOutput {
  stdout: string;
  stderr: string;
  exit_code: number;
  result: string;
  diagnostics?: Diagnostic[];
}

function readLine(): Promise<string | null> {
  return new Promise((resolve) => {
    let buffer = "";
    const stdin = process.stdin;
    stdin.setEncoding("utf-8");
    const onData = (chunk: string) => {
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

function writeOutput(output: EngineOutput): void {
  process.stdout.write(JSON.stringify(output) + "\n");
}

function writeError(message: string, exitCode = 1): void {
  writeOutput({
    stdout: "", stderr: message, exit_code: exitCode, result: "",
    diagnostics: [{ file: "", line: 0, col: 0, message, severity: "error" }],
  });
}

// ─── VLQ Base64 ─────────────────────────────────────────────────────────

const VLQ_CHARS = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

function vlqEncode(value: number): string {
  let result = "";
  let v = value < 0 ? ((-value) << 1) | 1 : value << 1;
  while (v >= 32) {
    result += VLQ_CHARS[(v & 31) | 32];
    v >>>= 5;
  }
  result += VLQ_CHARS[v];
  return result;
}

function generateSourceMap(tsSource: string, jsSource: string, filename: string): string {
  const tsLines = tsSource.split("\n");
  const jsLines = jsSource.split("\n");
  // Map each JS line to its closest TS line
  const segments: string[] = [];
  let srcLine = 0;
  for (let j = 0; j < jsLines.length; j++) {
    // Skip pure-generated lines (e.g., "use strict")
    if (j === 0 && jsLines[0].includes("use strict")) {
      segments.push(vlqEncode(0)); // column 0 -> srcLine 0, col 0, name 0
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
    names: [] as string[],
    mappings: segments.join(";"),
  };
  return Buffer.from(JSON.stringify(map)).toString("base64");
}

function appendSourceMap(js: string, tsSource: string, filename: string): string {
  const b64 = generateSourceMap(tsSource, js, filename);
  return `${js}\n//# sourceMappingURL=data:application/json;base64,${b64}`;
}

// ─── Transpile ───────────────────────────────────────────────────────────

function transpileTypeScript(code: string, filename = "input.ts"): { js: string; sourceMap?: string } {
  try {
    if (typeof Deno !== "undefined" && (Deno as any).transpileOnly) {
      const result = (Deno as any).transpileOnly({ [filename]: code }, {});
      const js = result?.[filename]?.source;
      if (js) return { js };
    }
  } catch { /* ignore */ }

  try {
    if (typeof require !== "undefined") {
      const esbuild = require("esbuild");
      const result = esbuild.transformSync(code, {
        loader: filename.endsWith(".tsx") ? "tsx" : "ts",
        target: "es2022",
        sourcemap: "inline",
      });
      if (result?.code) return { js: result.code, sourceMap: result.map };
    }
  } catch { /* ignore */ }

  const js = stripTypeAnnotations(code);
  return { js };
}

// ─── Regex-based TS Stripper ─────────────────────────────────────────────

const TS_PATTERNS: Array<[RegExp, string | ((...m: string[]) => string)]> = [
  [/:?\s*(?:number|string|boolean|void|null|undefined|any|never|unknown|bigint|symbol|object)\s*(?=[=,);\]}:]|$)/g, ""],
  [/:?\s*[A-Z][A-Za-z_$<>[\]\s,|&]*(?:\s*[|&]\s*[A-Z][A-Za-z_$<>[\]\s,|&]*)*\s*(?=[=,);\]}:]|$)/g, ""],
  [/\s+as\s+(?:const|[A-Za-z_$][\w$<>[\]\s,|&]*)/g, ""],
  [/\s+satisfies\s+[A-Za-z_$][\w$<>[\]\s,|&]*/g, ""],
  [/interface\s+\w+(?:\s*extends\s+[^{]+)?\s*\{[^}]*\}\s*/g, ""],
  [/type\s+\w+(?:<[^>]+>)?\s*=\s*[^;]+;/g, ""],
  [/\benum\s+(\w+)\s*\{([^}]*)\}/g, (_m: string, name: string, body: string) => {
    const entries = body.split(",").map((e: string) => {
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

function stripTypeAnnotations(code: string): string {
  let result = code;
  for (const [pattern, replacement] of TS_PATTERNS) {
    result = result.replace(pattern, replacement as string);
  }
  result = result.replace(/^\/\/\/\s*<reference\s+path\s*=.*\/>$/gm, "");
  result = result.replace(/^\/\/\/\s*<reference\s+types\s*=.*\/>$/gm, "");
  result = result.replace(/^import\s+['"][^'"]+['"];\s*$/gm, "");
  return result;
}

// ─── Type Checking via tsc ───────────────────────────────────────────────

async function runTsc(code: string, filename: string): Promise<Diagnostic[]> {
  const fs = await import("fs");
  const os = await import("os");
  const path = await import("path");
  const cp = require("child_process");

  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "klyron-tsc-"));
  const srcFile = path.join(tmpDir, filename || "input.ts");
  const configFile = path.join(tmpDir, "tsconfig.json");

  fs.writeFileSync(srcFile, code, "utf-8");
  fs.writeFileSync(configFile, JSON.stringify({
    compilerOptions: {
      target: "ES2022",
      module: "ESNext",
      strict: true,
      noEmit: true,
      skipLibCheck: true,
      forceConsistentCasingInFileNames: true,
    },
    include: [filename || "input.ts"],
  }, null, 2), "utf-8");

  const diags: Diagnostic[] = [];

  try {
    // Try npx tsc first, then direct tsc
    const tscCmds = [
      ["npx", "--yes", "typescript", "--project", configFile, "--noEmit", "--pretty", "false"],
      ["tsc", "--project", configFile, "--noEmit", "--pretty", "false"],
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

    const allOutput = stdout + "\n" + stderr;
    const lineRegex = /^(.+?)\((\d+),(\d+)\):\s+(error|warning)\s+(\S+):\s+(.+)$/gm;
    let m: RegExpExecArray | null;
    while ((m = lineRegex.exec(allOutput)) !== null) {
      diags.push({
        file: path.basename(m[1]),
        line: parseInt(m[2]),
        col: parseInt(m[3]),
        message: `${m[5]}: ${m[6]}`,
        severity: m[4],
      });
    }
  } catch (e: any) {
    if (diags.length === 0) {
      diags.push({ file: filename, line: 0, col: 0, message: `tsc check failed: ${e.message}`, severity: "error" });
    }
  } finally {
    try { fs.rmSync(tmpDir, { recursive: true, force: true }); } catch { /* ignore */ }
  }

  return diags;
}

// ─── Diagnostics Extraction ──────────────────────────────────────────────

function extractDiagnostics(err: Error, sourceTs?: string): Diagnostic[] {
  const msg = err.message;
  const lines = msg.split("\n");
  const diags: Diagnostic[] = [];
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

// ─── Execution ───────────────────────────────────────────────────────────

async function execCode(code: string, filename = "input.ts"): Promise<EngineOutput> {
  try {
    const { js } = transpileTypeScript(code, filename);
    const jsWithMap = appendSourceMap(js, code, filename);
    const asyncFn = new Function(`"use strict"; return (async () => { ${jsWithMap} })();`);
    const result = await asyncFn();
    return { stdout: String(result ?? ""), stderr: "", exit_code: 0, result: String(result ?? "") };
  } catch (e: unknown) {
    const err = e instanceof Error ? e : new Error(String(e));
    return {
      stdout: "", stderr: err.message, exit_code: 1, result: "",
      diagnostics: extractDiagnostics(err, code),
    };
  }
}

async function execFile(filename: string): Promise<EngineOutput> {
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
    const asyncFn = new Function(`"use strict"; return (async () => { ${jsWithMap} })();`);
    const result = await asyncFn();
    return { stdout: String(result ?? ""), stderr: "", exit_code: 0, result: String(result ?? "") };
  } catch (e: unknown) {
    const err = e instanceof Error ? e : new Error(String(e));
    return {
      stdout: "", stderr: err.message, exit_code: 1, result: "",
      diagnostics: extractDiagnostics(err),
    };
  }
}

async function evalExpr(expr: string): Promise<EngineOutput> {
  try {
    const js = stripTypeAnnotations(expr);
    const result = eval(js);
    return { stdout: String(result ?? ""), stderr: "", exit_code: 0, result: JSON.stringify(result) };
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : String(e);
    return { stdout: "", stderr: msg, exit_code: 1, result: "" };
  }
}

async function checkType(code: string, filename = "input.ts"): Promise<EngineOutput> {
  try {
    const diags = await runTsc(code, filename);
    const errors = diags.filter(d => d.severity === "error");
    return {
      stdout: "", stderr: "",
      exit_code: errors.length > 0 ? 1 : 0,
      result: errors.length > 0 ? "Type errors found" : "No type errors",
      diagnostics: diags,
    };
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : String(e);
    return { stdout: "", stderr: msg, exit_code: 1, result: "", diagnostics: [] };
  }
}

// ─── Main Loop ───────────────────────────────────────────────────────────

async function main() {
  process.stdin.setEncoding("utf-8");
  process.on("uncaughtException", (err) => writeError(String(err)));
  process.on("unhandledRejection", (reason) => writeError(String(reason)));

  while (true) {
    const line = await readLine();
    if (line === null) break;

    let input: EngineInput;
    try { input = JSON.parse(line); }
    catch { writeError("Invalid JSON input"); continue; }

    const { action, code, args, filename } = input;
    let output: EngineOutput;

    switch (action) {
      case "exec":
        output = await execCode(code ?? "", filename ?? "input.ts");
        break;
      case "file":
        output = await execFile(filename ?? args ?? "input.ts");
        break;
      case "eval":
        output = await evalExpr(code ?? "");
        break;
      case "check":
      case "typecheck":
        output = await checkType(code ?? "", filename ?? "input.ts");
        break;
      case "ping":
      case "":
        output = { stdout: "pong", stderr: "", exit_code: 0, result: "ok" };
        break;
      default:
        writeError(`Unknown action: ${action}`);
        continue;
    }
    writeOutput(output);
  }
}

main().catch((e) => writeError(String(e)));
