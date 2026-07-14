#!/usr/bin/env python3
import sys
import json
import ast
import traceback
import subprocess
import tempfile
import os
import signal
import shutil

MAX_OUTPUT = 1 << 20

def write_output(stdout="", stderr="", exit_code=0, result="", diagnostics=None):
    out = {
        "stdout": stdout,
        "stderr": stderr,
        "exit_code": exit_code,
        "result": result,
    }
    if diagnostics:
        out["diagnostics"] = diagnostics
    sys.stdout.write(json.dumps(out, ensure_ascii=False) + "\n")
    sys.stdout.flush()


def exec_code(code, files=None, filename=""):
    try:
        # Multi-file: write to temp dir and run as subprocess
        if files:
            tmpdir = tempfile.mkdtemp(prefix="klyron_py_")
            try:
                for f in files:
                    fpath = os.path.join(tmpdir, os.path.basename(f["name"]))
                    os.makedirs(os.path.dirname(fpath), exist_ok=True)
                    with open(fpath, "w") as fh:
                        fh.write(f["content"])
                if code:
                    entry = os.path.join(tmpdir, filename or "main.py")
                    with open(entry, "w") as fh:
                        fh.write(code)
                else:
                    entry = os.path.join(tmpdir, files[0]["name"])

                result = subprocess.run(
                    [sys.executable, entry],
                    capture_output=True, text=True, timeout=30
                )
                write_output(
                    stdout=result.stdout[:MAX_OUTPUT],
                    stderr=result.stderr[:MAX_OUTPUT],
                    exit_code=result.returncode,
                    result=result.stdout[:MAX_OUTPUT]
                )
            finally:
                shutil.rmtree(tmpdir, ignore_errors=True)
            return

        # Single code exec with stdout capture
        import io
        old_stdout = sys.stdout
        sys.stdout = captured = io.StringIO()
        try:
            compiled = compile(code, "<exec>", "exec")
            local_ns = {"__builtins__": __builtins__}
            exec(compiled, local_ns)
            stdout = captured.getvalue()
            result_val = local_ns.get("_", "")
            write_output(stdout=stdout, exit_code=0, result=json.dumps(result_val))
        finally:
            sys.stdout = old_stdout
    except SyntaxError as e:
        diag = [{"file": e.filename or "<eval>", "line": e.lineno or 0,
                 "col": e.offset or 0, "message": e.msg, "severity": "error"}]
        write_output(stderr=traceback.format_exc(), exit_code=1, diagnostics=diag)
    except Exception as e:
        write_output(stderr=traceback.format_exc(), exit_code=1)


def exec_file(filename):
    try:
        result = subprocess.run(
            [sys.executable, filename],
            capture_output=True, text=True, timeout=30
        )
        write_output(
            stdout=result.stdout[:MAX_OUTPUT],
            stderr=result.stderr[:MAX_OUTPUT],
            exit_code=result.returncode,
            result=result.stdout[:MAX_OUTPUT]
        )
    except subprocess.TimeoutExpired:
        write_output(stderr="Execution timed out", exit_code=124)
    except Exception as e:
        write_output(stderr=str(e), exit_code=1)


def eval_expr(expr):
    try:
        parsed = ast.parse(expr, mode="eval")
        compiled = compile(parsed, "<eval>", "eval")
        result = eval(compiled)
        write_output(result=json.dumps(result))
    except Exception as e:
        write_output(stderr=str(e), exit_code=1)


def check_code(code):
    try:
        ast.parse(code)
        write_output(result="No syntax errors")
    except SyntaxError as e:
        diag = [{
            "file": e.filename or "<eval>",
            "line": e.lineno or 0,
            "col": e.offset or 0,
            "message": e.msg,
            "severity": "error",
        }]
        write_output(stderr=e.msg, exit_code=1, result="Syntax error",
                     diagnostics=diag)


def pip_install(package, project=None):
    try:
        cmd = [sys.executable, "-m", "pip", "install", package]
        if project:
            cmd += ["--target", os.path.join(project, "vendor")]
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=120)
        write_output(
            stdout=result.stdout[:MAX_OUTPUT],
            stderr=result.stderr[:MAX_OUTPUT],
            exit_code=result.returncode,
            result="ok" if result.returncode == 0 else "failed"
        )
    except Exception as e:
        write_output(stderr=str(e), exit_code=1)


def main():
    signal.signal(signal.SIGPIPE, signal.SIG_IGN)

    while True:
        line = sys.stdin.readline()
        if not line:
            break

        line = line.strip()
        if not line:
            continue

        try:
            input_data = json.loads(line)
        except json.JSONDecodeError as e:
            write_output(stderr=f"Invalid JSON: {e}", exit_code=1)
            continue

        action = input_data.get("action", "")
        code = input_data.get("code", "")
        args = input_data.get("args", "")
        filename = input_data.get("filename", "")
        files = input_data.get("files", [])

        if action in ("exec", "run"):
            exec_code(code, files, filename)
        elif action == "file":
            exec_file(filename or args or code)
        elif action == "eval":
            eval_expr(code)
        elif action in ("check", "typecheck"):
            check_code(code)
        elif action == "pip":
            pip_install(args, input_data.get("project"))
        elif action in ("ping", ""):
            write_output(stdout="pong", result="ok")
        else:
            write_output(stderr=f"Unknown action: {action}", exit_code=1)


if __name__ == "__main__":
    main()
