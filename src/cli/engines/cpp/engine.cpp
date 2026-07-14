#include <iostream>
#include <string>
#include <cstdlib>
#include <cstring>
#include <cstdio>
#include <unistd.h>
#include <sys/wait.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <memory>
#include <sstream>
#include <vector>
#include <algorithm>
#include <functional>
#include <system_error>

namespace {

constexpr size_t MAX_OUTPUT = 1 << 20;
constexpr int COMPILE_TIMEOUT = 120;
constexpr int RUN_TIMEOUT = 30;
constexpr int MAX_FILES = 256;

// ─── RAII File Descriptor ─────────────────────────────────────────────

struct FdCloser {
  void operator()(int *fd) const { if (fd && *fd >= 0) close(*fd); delete fd; }
};
using FdPtr = std::unique_ptr<int, FdCloser>;

// ─── JSON String Builder ───────────────────────────────────────────────

class JsonWriter {
  std::string buf_;
public:
  void raw(const std::string &s) { buf_ += s; }
  void raw(const char *s) { buf_ += s; }

  void key(const char *k) {
    if (buf_.empty()) buf_ += '{';
    else if (buf_.back() != '{') buf_ += ',';
    buf_ += '"'; buf_ += k; buf_ += "\":";
  }

  void str(const std::string &s) {
    buf_ += '"';
    for (unsigned char c : s) {
      switch (c) {
        case '"': buf_ += "\\\""; break;
        case '\\': buf_ += "\\\\"; break;
        case '\n': buf_ += "\\n"; break;
        case '\r': buf_ += "\\r"; break;
        case '\t': buf_ += "\\t"; break;
        default:
          if (c < 0x20) { char x[8]; snprintf(x, sizeof(x), "\\u%04x", c); buf_ += x; }
          else buf_ += c;
      }
    }
    buf_ += '"';
  }

  void num(int n) { buf_ += std::to_string(n); }
  void num(unsigned long n) { buf_ += std::to_string(n); }

  void flush() { buf_ += '}'; buf_ += '\n'; write(STDOUT_FILENO, buf_.data(), buf_.size()); buf_.clear(); }

  std::string &buf() { return buf_; }
};

// ─── JSON Token Reader (recursive descent) ─────────────────────────────

class JsonReader {
  const char *p_;
public:
  explicit JsonReader(const std::string &s) : p_(s.data()) {}

  void skip_ws() { while (*p_ == ' ' || *p_ == '\t' || *p_ == '\n' || *p_ == '\r') p_++; }

  bool maybe_match(const char *key) {
    skip_ws();
    const char *saved = p_;
    if (*p_ != '"') return false;
    p_++;
    while (*p_ && *p_ != '"') { if (*p_ == '\\') p_++; p_++; }
    if (*p_ != '"') return false;
    size_t klen = strlen(key);
    if ((size_t)(p_ - saved - 1) != klen || strncmp(saved + 1, key, klen) != 0) { p_ = saved; return false; }
    p_++;
    return true;
  }

  bool match_key(const char *key) {
    const char *saved = p_;
    skip_ws(); while (*p_ == ',' || *p_ == '{') p_++;
    if (!maybe_match(key)) { p_ = saved; return false; }
    skip_ws();
    if (*p_ != ':') { p_ = saved; return false; }
    p_++;
    return true;
  }

  std::string read_string() {
    skip_ws();
    if (*p_ != '"') return {};
    p_++;
    std::string out;
    while (*p_ && *p_ != '"') {
      if (*p_ == '\\') {
        p_++;
        switch (*p_) {
          case '"': out += '"'; break;
          case '\\': out += '\\'; break;
          case 'n': out += '\n'; break;
          case 'r': out += '\r'; break;
          case 't': out += '\t'; break;
          default: out += *p_; break;
        }
      } else {
        out += *p_;
      }
      p_++;
    }
    if (*p_ == '"') p_++;
    return out;
  }

  std::string read_raw_value() {
    skip_ws();
    const char *start = p_;
    if (*p_ == '"') {
      p_++;
      while (*p_) {
        if (*p_ == '"') { p_++; break; }
        if (*p_ == '\\') p_++;
        p_++;
      }
      return std::string(start, p_ - start);
    }
    if (*p_ == '[' || *p_ == '{') {
      char open = *p_, close = (*p_ == '[') ? ']' : '}';
      int depth = 1; p_++;
      while (*p_ && depth > 0) {
        if (*p_ == open) depth++;
        if (*p_ == close) depth--;
        if (*p_ == '"') { p_++; while (*p_ && !(*p_ == '"' && *(p_-1) != '\\')) p_++; if (*p_) p_++; }
        else p_++;
      }
      return std::string(start, p_ - start);
    }
    while (*p_ && *p_ != ',' && *p_ != '}' && *p_ != ']' && *p_ != '\n') p_++;
    return std::string(start, p_ - start);
  }

  std::vector<std::pair<std::string, std::string>> parse_files() {
    std::vector<std::pair<std::string, std::string>> files;
    skip_ws();
    if (*p_ != '[') return files;
    p_++;
    while (*p_ && *p_ != ']') {
      skip_ws();
      if (*p_ != '{') break;
      p_++;
      std::string name, content;
      while (*p_ && *p_ != '}') {
        if (match_key("name")) name = read_string();
        else if (match_key("content")) content = read_string();
        else p_++;
      }
      if (*p_ == '}') p_++;
      if (!name.empty()) files.emplace_back(std::move(name), std::move(content));
      skip_ws();
      if (*p_ == ',') p_++;
    }
    if (*p_ == ']') p_++;
    return files;
  }
};

// ─── Compiler Argument Parser ──────────────────────────────────────────

struct ParsedArgs {
  std::string standard = "c++20";
  std::string optimization = "-O2";
  std::vector<std::string> warnings;
  std::vector<std::string> libraries;
  std::vector<std::string> include_dirs;
  std::vector<std::string> defines;
  bool debug = false;
  std::vector<std::string> sanitizers;
  bool lto = false;
  bool modules = false;
  bool modules_ts = false;
  std::vector<std::string> extra;
};

static ParsedArgs parse_args(const std::string &args) {
  ParsedArgs pa;
  std::istringstream ss(args);
  std::string tok;
  while (ss >> tok) {
    if ((tok.rfind("--std=", 0) == 0 || tok.rfind("-std=", 0) == 0)) {
      pa.standard = tok.substr(tok.find('=') + 1);
    } else if (tok.size() >= 2 && tok.size() <= 6 && tok[0] == '-' && tok[1] == 'O') {
      pa.optimization = tok;
    } else if (tok.rfind("-W", 0) == 0) {
      pa.warnings.push_back(tok);
    } else if (tok.rfind("-l", 0) == 0 && tok.size() > 2) {
      pa.libraries.push_back(tok);
    } else if (tok.rfind("-I", 0) == 0 && tok.size() > 2) {
      pa.include_dirs.push_back(tok);
    } else if (tok.rfind("-D", 0) == 0 && tok.size() > 2) {
      pa.defines.push_back(tok);
    } else if (tok == "-g") {
      pa.debug = true;
    } else if (tok.rfind("-fsanitize=", 0) == 0) {
      pa.sanitizers.push_back(tok);
    } else if (tok == "-flto") {
      pa.lto = true;
    } else if (tok == "-fmodules") {
      pa.modules = true;
    } else if (tok == "-fmodules-ts") {
      pa.modules_ts = true;
    } else {
      pa.extra.push_back(tok);
    }
  }
  return pa;
}

static bool standard_has_coroutines(const std::string &std) {
  return std == "c++20" || std == "c++23" || std == "c++26" ||
         std == "gnu++20" || std == "gnu++23" || std == "gnu++26" ||
         std == "c++2a" || std == "c++2b";
}

// ─── Subprocess ────────────────────────────────────────────────────────

struct ProcResult {
  int exit_code;
  std::string out;
  std::string err;
};

static ProcResult exec_shell(const std::string &cmd, int timeout_sec,
                              const std::string &stdin_data = "") {
  int po[2], pe[2], pi[2];
  bool has_stdin = !stdin_data.empty();
  if (pipe(po) < 0 || pipe(pe) < 0) return {-1, "", "pipe() failed"};
  if (has_stdin && pipe(pi) < 0) {
    close(po[0]); close(po[1]); close(pe[0]); close(pe[1]);
    return {-1, "", "pipe() failed"};
  }

  for (int i = 3; i < 256; i++)
    if (i != po[0] && i != po[1] && i != pe[0] && i != pe[1] &&
        (!has_stdin || (i != pi[0] && i != pi[1])))
      fcntl(i, F_SETFD, FD_CLOEXEC);

  struct sigaction old_sa;
  memset(&old_sa, 0, sizeof(old_sa));
  struct sigaction dfl_sa; memset(&dfl_sa, 0, sizeof(dfl_sa)); dfl_sa.sa_handler = SIG_DFL;
  sigaction(SIGCHLD, &dfl_sa, &old_sa);

  pid_t pid = fork();
  if (pid == 0) {
    struct sigaction dfl; memset(&dfl, 0, sizeof(dfl)); dfl.sa_handler = SIG_DFL;
    sigaction(SIGPIPE, &dfl, nullptr);
    close(po[0]); close(pe[0]);
    dup2(po[1], STDOUT_FILENO); dup2(pe[1], STDERR_FILENO);
    close(po[1]); close(pe[1]);
    if (has_stdin) {
      dup2(pi[0], STDIN_FILENO);
      close(pi[0]); close(pi[1]);
    }
    if (timeout_sec > 0) alarm(timeout_sec);
    execl("/bin/sh", "sh", "-c", cmd.c_str(), (char *)nullptr);
    _exit(127);
  }

  close(po[1]); close(pe[1]);
  if (has_stdin) close(pi[0]);

  if (has_stdin) {
    size_t written = 0;
    while (written < stdin_data.size()) {
      ssize_t n = write(pi[1], stdin_data.data() + written, stdin_data.size() - written);
      if (n > 0) written += n;
      else break;
    }
    close(pi[1]);
  }

  int fl_o = fcntl(po[0], F_GETFL, 0);
  int fl_e = fcntl(pe[0], F_GETFL, 0);
  fcntl(po[0], F_SETFL, fl_o | O_NONBLOCK);
  fcntl(pe[0], F_SETFL, fl_e | O_NONBLOCK);

  std::string out, err;
  char buf[65536];
  struct timeval tv = {0, 50000};
  bool active = true;

  while (active) {
    fd_set rfds;
    FD_ZERO(&rfds);
    FD_SET(po[0], &rfds); FD_SET(pe[0], &rfds);
    int maxfd = std::max(po[0], pe[0]);
    int ret;
    do { ret = select(maxfd + 1, &rfds, nullptr, nullptr, &tv); } while (ret < 0 && errno == EINTR);
    if (ret < 0) break;
    active = false;
    if (FD_ISSET(po[0], &rfds)) {
      ssize_t n = read(po[0], buf, sizeof(buf));
      if (n > 0) { out.append(buf, n); active = true; }
    }
    if (FD_ISSET(pe[0], &rfds)) {
      ssize_t n = read(pe[0], buf, sizeof(buf));
      if (n > 0) { err.append(buf, n); active = true; }
    }
  }

  close(po[0]); close(pe[0]);

  int status = -1;
  int wret;
  do { wret = waitpid(pid, &status, 0); } while (wret < 0 && errno == EINTR);

  sigaction(SIGCHLD, &old_sa, nullptr);

  if (wret < 0) return {-1, out, err};
  int ec = WIFEXITED(status) ? WEXITSTATUS(status) :
           WIFSIGNALED(status) ? 128 + WTERMSIG(status) : -1;
  return {ec, out, err};
}

static std::string detect_compiler() {
  for (const char *c : {"g++", "clang++", "c++"}) {
    std::string cmd = std::string(c) + " --version >/dev/null 2>&1";
    if (system(cmd.c_str()) == 0) return c;
  }
  return "g++";
}

struct SourceFile { std::string name; std::string content; };

static bool write_sources(const std::string &dir, const std::vector<SourceFile> &files,
                           const std::string &code, const std::string &fname) {
  auto write_one = [&](const std::string &path, const std::string &content) -> bool {
    FILE *fp = fopen(path.c_str(), "w");
    if (!fp) return false;
    fputs(content.c_str(), fp);
    fclose(fp);
    return true;
  };
  if (!files.empty()) {
    for (auto &f : files)
      if (!write_one(dir + "/" + f.name, f.content)) return false;
    return true;
  }
  std::string path = dir + "/" + (fname.empty() ? "main.cpp" : fname);
  return write_one(path, code);
}

static std::string find_sources(const std::string &dir) {
  std::string result;
  std::string fcmd = "find " + dir + " -maxdepth 1 \\( -name '*.cpp' -o -name '*.cxx' "
    "-o -name '*.cc' -o -name '*.ixx' -o -name '*.hpp' -o -name '*.hxx' \\) 2>/dev/null";
  FILE *fp = popen(fcmd.c_str(), "r");
  if (!fp) return dir + "/main.cpp";
  char line[4096];
  while (fgets(line, sizeof(line), fp)) {
    size_t l = strlen(line);
    while (l > 0 && (line[l-1] == '\n' || line[l-1] == '\r')) line[--l] = '\0';
    if (l == 0) continue;
    if (!result.empty()) result += ' ';
    result += line;
  }
  pclose(fp);
  return result.empty() ? dir + "/main.cpp" : result;
}

// ─── Build Compile Command ─────────────────────────────────────────────

static std::string build_compile_cmd(const std::string &compiler,
                                      const std::string &tmpdir,
                                      const std::string &src_list,
                                      const ParsedArgs &pa) {
  std::string cmd = compiler + " -std=" + pa.standard + " -o " + tmpdir + "/prog " + src_list;

  cmd += " " + pa.optimization;

  for (const auto &w : pa.warnings) { cmd += " "; cmd += w; }
  for (const auto &idir : pa.include_dirs) { cmd += " "; cmd += idir; }
  for (const auto &d : pa.defines) { cmd += " "; cmd += d; }

  if (pa.debug) cmd += " -g";

  for (const auto &s : pa.sanitizers) { cmd += " "; cmd += s; }

  if (pa.lto) cmd += " -flto";
  if (pa.modules) cmd += " -fmodules";
  if (pa.modules_ts) cmd += " -fmodules-ts";

  if (standard_has_coroutines(pa.standard)) cmd += " -fcoroutines";

  for (const auto &e : pa.extra) { cmd += " "; cmd += e; }

  for (const auto &l : pa.libraries) { cmd += " "; cmd += l; }

  cmd += " -lm -pthread 2>&1";
  return cmd;
}

// ─── Main Logic ────────────────────────────────────────────────────────

static void handle_action(const std::string &action, const std::string &code,
                           const std::string &args, const std::string &files_raw,
                           const std::string &filename,
                           const std::string &stdin_data) {
  JsonWriter w;
  ParsedArgs pa = parse_args(args);

  auto compile_and_run = [&](const std::string &src, bool compile_only,
                               const std::vector<SourceFile> &extra_files) {
    char tdir[] = "/tmp/klyron_cpp_XXXXXX";
    if (!mkdtemp(tdir)) {
      w.key("stderr"); w.str("Failed to create temp dir");
      w.key("exit_code"); w.num(1); w.key("result"); w.str(""); w.flush();
      return;
    }
    std::string tmpdir(tdir);
    auto cleanup = [&]() { exec_shell("rm -rf " + tmpdir, 5); };
    struct ScopeGuard { std::function<void()> fn; ~ScopeGuard() { fn(); } } guard{cleanup};

    if (!write_sources(tmpdir, extra_files, src, filename)) {
      w.key("stderr"); w.str("Failed to write source file");
      w.key("exit_code"); w.num(1); w.key("result"); w.str(""); w.flush();
      return;
    }

    std::string src_list = find_sources(tmpdir);
    std::string compiler = detect_compiler();
    std::string compile_cmd = build_compile_cmd(compiler, tmpdir, src_list, pa);

    auto cr = exec_shell(compile_cmd, COMPILE_TIMEOUT);
    if (cr.exit_code != 0) {
      w.key("stdout"); w.str(cr.out);
      w.key("stderr"); w.str(cr.err);
      w.key("exit_code"); w.num(cr.exit_code);
      w.key("result"); w.str("Compilation failed");
      w.flush();
      return;
    }

    if (compile_only) {
      w.key("stdout"); w.str("Compiled successfully");
      w.key("exit_code"); w.num(0); w.key("result"); w.str("ok"); w.flush();
      return;
    }

    std::string run_cmd = tmpdir + "/prog";
    if (!pa.extra.empty()) {
      for (const auto &e : pa.extra) { run_cmd += " "; run_cmd += e; }
    }
    auto rr = exec_shell(run_cmd, RUN_TIMEOUT, stdin_data);
    w.key("stdout"); w.str(rr.out);
    w.key("stderr"); w.str(rr.err);
    w.key("exit_code"); w.num(rr.exit_code);
    w.key("result"); w.str(rr.out);
    w.flush();
  };

  std::vector<SourceFile> extra_files;
  if (!files_raw.empty()) {
    JsonReader jr(files_raw);
    auto pairs = jr.parse_files();
    for (auto &p : pairs) extra_files.push_back({std::move(p.first), std::move(p.second)});
  }

  if (action == "exec" || action == "run") {
    if (code.empty() && extra_files.empty()) {
      w.key("stderr"); w.str("No code provided");
      w.key("exit_code"); w.num(1); w.key("result"); w.str(""); w.flush();
      return;
    }
    compile_and_run(code, false, extra_files);
  } else if (action == "compile") {
    if (code.empty() && extra_files.empty()) {
      w.key("stderr"); w.str("No code provided");
      w.key("exit_code"); w.num(1); w.key("result"); w.str(""); w.flush();
      return;
    }
    compile_and_run(code, true, extra_files);
  } else if (action == "eval") {
    if (code.empty()) {
      w.key("stderr"); w.str("No expression provided");
      w.key("exit_code"); w.num(1); w.key("result"); w.str(""); w.flush();
      return;
    }
    std::string wrapped =
      "#include <iostream>\n#include <cmath>\n"
      "int main() {\n  std::cout << (" + code + ") << std::endl;\n  return 0;\n}\n";
    compile_and_run(wrapped, false, {});
  } else if (action == "format") {
    if (code.empty()) {
      w.key("stderr"); w.str("No code provided");
      w.key("exit_code"); w.num(1); w.key("result"); w.str(""); w.flush();
      return;
    }
    char tdir[] = "/tmp/klyron_cpp_XXXXXX";
    if (!mkdtemp(tdir)) {
      w.key("stderr"); w.str("Failed to create temp dir");
      w.key("exit_code"); w.num(1); w.key("result"); w.str(""); w.flush();
      return;
    }
    std::string tmpdir(tdir);
    auto cleanup = [&]() { exec_shell("rm -rf " + tmpdir, 5); };
    struct ScopeGuard { std::function<void()> fn; ~ScopeGuard() { fn(); } } guard{cleanup};
    if (!write_sources(tmpdir, {}, code, "input.cpp")) {
      w.key("stderr"); w.str("Failed to write source file");
      w.key("exit_code"); w.num(1); w.key("result"); w.str(""); w.flush();
      return;
    }
    auto fr = exec_shell("clang-format " + tmpdir + "/input.cpp", 10);
    w.key("stdout"); w.str(fr.out);
    w.key("stderr"); w.str(fr.err);
    w.key("exit_code"); w.num(fr.exit_code);
    w.key("result"); w.str(fr.exit_code == 0 ? "ok" : "Format failed");
    w.flush();
  } else if (action == "lint") {
    if (code.empty()) {
      w.key("stderr"); w.str("No code provided");
      w.key("exit_code"); w.num(1); w.key("result"); w.str(""); w.flush();
      return;
    }
    char tdir[] = "/tmp/klyron_cpp_XXXXXX";
    if (!mkdtemp(tdir)) {
      w.key("stderr"); w.str("Failed to create temp dir");
      w.key("exit_code"); w.num(1); w.key("result"); w.str(""); w.flush();
      return;
    }
    std::string tmpdir(tdir);
    auto cleanup = [&]() { exec_shell("rm -rf " + tmpdir, 5); };
    struct ScopeGuard { std::function<void()> fn; ~ScopeGuard() { fn(); } } guard{cleanup};
    std::string wrapped = "#include <iostream>\n#include <vector>\n#include <string>\n#include <algorithm>\n" + code;
    if (!write_sources(tmpdir, {}, wrapped, "input.cpp")) {
      w.key("stderr"); w.str("Failed to write source file");
      w.key("exit_code"); w.num(1); w.key("result"); w.str(""); w.flush();
      return;
    }
    std::string lint_out;
    int lint_exit;
    auto lr = exec_shell("clang-tidy " + tmpdir + "/input.cpp 2>&1", 30);
    if (lr.out.find("not found") == std::string::npos) {
      lint_out = lr.out;
      lint_exit = lr.exit_code;
    } else {
      auto cr = exec_shell("cppcheck --enable=all --language=c++ " + tmpdir + "/input.cpp 2>&1", 30);
      if (cr.out.find("not found") == std::string::npos) {
        lint_out = cr.out;
        lint_exit = cr.exit_code;
      } else {
        lint_out = "No linter available (tried clang-tidy and cppcheck)";
        lint_exit = 1;
      }
    }
    w.key("stdout"); w.str("");
    w.key("stderr"); w.str(lint_out);
    w.key("exit_code"); w.num(lint_exit);
    w.key("result"); w.str(lint_exit == 0 ? "ok" : "Lint issues found");
    w.flush();
  } else if (action == "ping" || action.empty()) {
    w.key("stdout"); w.str("pong");
    w.key("exit_code"); w.num(0); w.key("result"); w.str("ok"); w.flush();
  } else {
    w.key("stderr"); w.str("Unknown action: " + action);
    w.key("exit_code"); w.num(1); w.key("result"); w.str(""); w.flush();
  }
}

} // anonymous namespace

int main() {
  setvbuf(stdout, nullptr, _IONBF, 0);
  setvbuf(stderr, nullptr, _IONBF, 0);

  struct sigaction sa;
  memset(&sa, 0, sizeof(sa));
  sa.sa_handler = SIG_IGN;
  sa.sa_flags = SA_NOCLDWAIT;
  sigaction(SIGPIPE, &sa, nullptr);
  sigaction(SIGCHLD, &sa, nullptr);

  std::string line;
  while (std::getline(std::cin, line)) {
    auto nl = line.find_last_not_of("\r\n");
    if (nl == std::string::npos) continue;
    line.resize(nl + 1);
    if (line.empty()) continue;

    JsonReader jr(line);
    jr.match_key("action");   std::string action = jr.read_string();
    jr.match_key("code");     std::string code = jr.read_string();
    jr.match_key("args");     std::string args = jr.read_string();
    jr.match_key("filename"); std::string filename = jr.read_string();

    std::string stdin_data;
    {
      JsonReader jr2(line);
      if (jr2.match_key("stdin")) stdin_data = jr2.read_string();
    }

    std::string files_raw;
    {
      JsonReader jr2(line);
      jr2.match_key("files"); files_raw = jr2.read_raw_value();
    }

    handle_action(action, code, args, files_raw, filename, stdin_data);
  }
  return 0;
}
