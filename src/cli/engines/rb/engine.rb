#!/usr/bin/env ruby
require 'json'
require 'tempfile'
require 'fileutils'
require 'open3'

MAX_OUTPUT = 1 << 20

$stdout.sync = true
$stderr.sync = true

def write_output(stdout: "", stderr: "", exit_code: 0, result: "", diagnostics: nil)
  out = {
    stdout: stdout,
    stderr: stderr,
    exit_code: exit_code,
    result: result,
  }
  out[:diagnostics] = diagnostics if diagnostics
  $stdout.puts(JSON.generate(out))
  $stdout.flush
end

def exec_code(code, files = nil, filename = "")
  if files && !files.empty?
    tmpdir = Dir.mktmpdir("klyron_rb_")
    begin
      files.each do |f|
        fpath = File.join(tmpdir, File.basename(f["name"]))
        FileUtils.mkdir_p(File.dirname(fpath))
        File.write(fpath, f["content"])
      end
      if code && !code.empty?
        entry = File.join(tmpdir, filename.empty? ? "main.rb" : filename)
        File.write(entry, code)
      else
        entry = File.join(tmpdir, files.first["name"])
      end

      stdout, stderr, status = Open3.capture3("ruby", entry)
      write_output(
        stdout: stdout[0, MAX_OUTPUT],
        stderr: stderr[0, MAX_OUTPUT],
        exit_code: status.exitstatus,
        result: stdout[0, MAX_OUTPUT]
      )
    ensure
      FileUtils.rm_rf(tmpdir)
    end
    return
  end

  # Captured execution
  begin
    result = eval(code, TOPLEVEL_BINDING)
    write_output(result: result.inspect)
  rescue SyntaxError => e
    diag = [{ file: "<eval>", line: e.lineno&.to_i || 0, col: 0,
              message: e.message, severity: "error" }]
    write_output(stderr: "#{e.class}: #{e.message}", exit_code: 1, diagnostics: diag)
  rescue => e
    write_output(stderr: "#{e.class}: #{e.message}\n#{e.backtrace&.first(10)&.join("\n")}", exit_code: 1)
  end
end

def exec_file(filename)
  begin
    stdout, stderr, status = Open3.capture3("ruby", filename)
    write_output(
      stdout: stdout[0, MAX_OUTPUT],
      stderr: stderr[0, MAX_OUTPUT],
      exit_code: status.exitstatus,
      result: stdout[0, MAX_OUTPUT]
    )
  rescue => e
    write_output(stderr: "#{e.class}: #{e.message}", exit_code: 1)
  end
end

def eval_expr(expr)
  begin
    result = eval(expr, TOPLEVEL_BINDING)
    write_output(result: JSON.generate(result))
  rescue => e
    write_output(stderr: e.message, exit_code: 1)
  end
end

def check_code(code)
  begin
    RubyVM::InstructionSequence.compile(code)
    write_output(result: "No syntax errors")
  rescue SyntaxError => e
    diag = [{ file: "<eval>", line: e.lineno&.to_i || 0, col: 0,
              message: e.message, severity: "error" }]
    write_output(stderr: e.message, exit_code: 1, result: "Syntax error", diagnostics: diag)
  end
end

def bundle_exec(args, project = nil)
  dir = project || Dir.pwd
  begin
    stdout, stderr, status = Open3.capture3("bundle", "exec", *args.split, chdir: dir)
    write_output(
      stdout: stdout[0, MAX_OUTPUT],
      stderr: stderr[0, MAX_OUTPUT],
      exit_code: status.exitstatus,
      result: status.success? ? "ok" : "failed"
    )
  rescue => e
    write_output(stderr: e.message, exit_code: 1)
  end
end

Signal.trap("PIPE", "IGNORE")

while line = $stdin.gets
  line = line.strip
  next if line.empty?

  begin
    input = JSON.parse(line)
  rescue JSON::ParserError => e
    write_output(stderr: "Invalid JSON: #{e.message}", exit_code: 1)
    next
  end

  action = input["action"] || ""
  code = input["code"] || ""
  args = input["args"] || ""
  filename = input["filename"] || ""
  files = input["files"] || []
  project = input["project"]

  case action
  when "exec", "run"
    exec_code(code, files, filename)
  when "file"
    exec_file(filename.empty? ? (args.empty? ? code : args) : filename)
  when "eval"
    eval_expr(code)
  when "check", "typecheck"
    check_code(code)
  when "bundle"
    bundle_exec(args, project)
  when "ping", ""
    write_output(stdout: "pong", result: "ok")
  else
    write_output(stderr: "Unknown action: #{action}", exit_code: 1)
  end
end
