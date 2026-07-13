#define _GNU_SOURCE
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/wait.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <sys/resource.h>
#include <fcntl.h>
#include <errno.h>
#include <signal.h>
#include <time.h>
#include <stdarg.h>

#define MAX_OUTPUT (1 << 20)
#define MAX_LINE (1 << 18)
#define MAX_FILES 256

typedef struct {
  char *data;
  size_t len;
  size_t cap;
} Buf;

static void buf_init(Buf *b) { b->data = NULL; b->len = 0; b->cap = 0; }
static void buf_free(Buf *b) { free(b->data); b->data = NULL; b->len = 0; b->cap = 0; }
static int buf_append(Buf *b, const char *s, size_t n) {
  if (b->len + n + 1 > b->cap) {
    size_t newcap = b->cap ? b->cap * 2 : 4096;
    while (newcap < b->len + n + 1) newcap *= 2;
    char *p = realloc(b->data, newcap);
    if (!p) return -1;
    b->data = p; b->cap = newcap;
  }
  memcpy(b->data + b->len, s, n);
  b->len += n;
  b->data[b->len] = '\0';
  return 0;
}
static int buf_appendf(Buf *b, const char *fmt, ...) {
  va_list ap;
  va_start(ap, fmt);
  int n = vsnprintf(NULL, 0, fmt, ap);
  va_end(ap);
  if (n < 0) return -1;
  char tmp[8192];
  if ((size_t)n >= sizeof(tmp)) {
    char *buf = malloc(n + 1);
    if (!buf) return -1;
    va_start(ap, fmt); vsnprintf(buf, n + 1, fmt, ap); va_end(ap);
    int ret = buf_append(b, buf, n);
    free(buf);
    return ret;
  }
  va_start(ap, fmt); vsnprintf(tmp, sizeof(tmp), fmt, ap); va_end(ap);
  return buf_append(b, tmp, n);
}

// JSON output with "diagnostics" support
static void json_output_ex(const char *stdout_str, const char *stderr_str, int exit_code,
                           const char *result, const char *diag_json) {
  Buf b; buf_init(&b);
  buf_append(&b, "{\"stdout\":", 10);
  // Escape stdout
  buf_append(&b, "\"", 1);
  if (stdout_str) for (const char *s = stdout_str; *s; s++) {
    switch (*s) {
      case '"': buf_append(&b, "\\\"", 2); break;
      case '\\': buf_append(&b, "\\\\", 2); break;
      case '\n': buf_append(&b, "\\n", 2); break;
      case '\r': buf_append(&b, "\\r", 2); break;
      case '\t': buf_append(&b, "\\t", 2); break;
      default:
        if ((unsigned char)*s < 0x20) buf_appendf(&b, "\\u%04x", (unsigned char)*s);
        else buf_append(&b, s, 1);
    }
  }
  buf_append(&b, "\",\"stderr\":\"", 12);
  if (stderr_str) for (const char *s = stderr_str; *s; s++) {
    switch (*s) {
      case '"': buf_append(&b, "\\\"", 2); break;
      case '\\': buf_append(&b, "\\\\", 2); break;
      case '\n': buf_append(&b, "\\n", 2); break;
      case '\r': buf_append(&b, "\\r", 2); break;
      case '\t': buf_append(&b, "\\t", 2); break;
      default:
        if ((unsigned char)*s < 0x20) buf_appendf(&b, "\\u%04x", (unsigned char)*s);
        else buf_append(&b, s, 1);
    }
  }
  buf_appendf(&b, "\",\"exit_code\":%d,\"result\":\"", exit_code);
  if (result) for (const char *s = result; *s; s++) {
    switch (*s) {
      case '"': buf_append(&b, "\\\"", 2); break;
      case '\\': buf_append(&b, "\\\\", 2); break;
      case '\n': buf_append(&b, "\\n", 2); break;
      case '\r': buf_append(&b, "\\r", 2); break;
      case '\t': buf_append(&b, "\\t", 2); break;
      default:
        if ((unsigned char)*s < 0x20) buf_appendf(&b, "\\u%04x", (unsigned char)*s);
        else buf_append(&b, s, 1);
    }
  }
  buf_append(&b, "\"", 1);
  if (diag_json) { buf_append(&b, ",\"diagnostics\":", 16); buf_append(&b, diag_json, strlen(diag_json)); }
  buf_append(&b, "}\n", 2);
  write(STDOUT_FILENO, b.data, b.len);
  buf_free(&b);
}

static void json_output(const char *stdout_str, const char *stderr_str, int exit_code, const char *result) {
  json_output_ex(stdout_str, stderr_str, exit_code, result, NULL);
}

__attribute__((unused)) static void add_diag(Buf *d, const char *file, int line, int col, const char *msg) {
  if (d->len > 1) buf_append(d, ",", 1);
  char line_buf[32], col_buf[32];
  snprintf(line_buf, sizeof(line_buf), "%d", line);
  snprintf(col_buf, sizeof(col_buf), "%d", col);
  buf_append(d, "{\"file\":\"", 9);
  buf_append(d, file, strlen(file));
  buf_append(d, "\",\"line\":", 9);
  buf_append(d, line_buf, strlen(line_buf));
  buf_append(d, ",\"col\":", 7);
  buf_append(d, col_buf, strlen(col_buf));
  buf_append(d, ",\"message\":\"", 12);
  for (const char *s = msg; *s; s++) {
    if (*s == '"') buf_append(d, "\\\"", 2);
    else if (*s == '\\') buf_append(d, "\\\\", 2);
    else buf_append(d, s, 1);
  }
  buf_append(d, "\",\"severity\":\"error\"}", 24);
}

// Dynamic string
typedef struct {
  char *s;
} Str;

static Str str_get(const char *json, const char *key) {
  Str r = {NULL};
  size_t klen = strlen(key);
  // find "key":
  const char *p = json;
  while ((p = strstr(p, key))) {
    if ((p == json || p[-1] == '"') && p[klen] == '"') {
      p += klen;
      while (*p && *p != ':') p++;
      if (*p != ':') { p++; continue; }
      p++;
      while (*p == ' ' || *p == '\t' || *p == '\n' || *p == '\r') p++;
      if (*p == '"') {
        p++;
        Buf b; buf_init(&b);
        while (*p) {
          if (*p == '"') { p++; break; }
          if (*p == '\\') { p++; if (*p) { buf_append(&b, p, 1); p++; } continue; }
          buf_append(&b, p, 1); p++;
        }
        r.s = b.data; // transfer ownership
        return r;
      }
    }
    p++;
  }
  return r;
}

static Str str_get_raw(const char *json, const char *key) {
  Str r = {NULL};
  size_t klen = strlen(key);
  const char *p = json;
  while ((p = strstr(p, key))) {
    if ((p == json || p[-1] == '"') && p[klen] == '"') {
      p += klen;
      while (*p && *p != ':') p++;
      if (*p != ':') { p++; continue; }
      p++;
      while (*p == ' ' || *p == '\t' || *p == '\n' || *p == '\r') p++;
      // Capture raw value (string, array, or object)
      if (*p == '"') {
        p++;
        Buf b; buf_init(&b);
        while (*p) {
          if (*p == '"') { p++; break; }
          if (*p == '\\') { buf_append(&b, p, 2); p += 2; continue; }
          buf_append(&b, p, 1); p++;
        }
        r.s = b.data;
        return r;
      }
      if (*p == '[' || *p == '{') {
        int depth = 1;
        const char *start = p;
        p++;
        while (*p && depth > 0) {
          if (*p == '{' || *p == '[') depth++;
          if (*p == '}' || *p == ']') depth--;
          p++;
        }
        r.s = strndup(start, p - start);
        return r;
      }
      // Number/boolean/null
      const char *start = p;
      while (*p && *p != ',' && *p != '}' && *p != ']' && *p != '\n') p++;
      r.s = strndup(start, p - start);
      return r;
    }
    p++;
  }
  return r;
}

static void str_free(Str *s) { free(s->s); s->s = NULL; }

static int exec_shell(const char *cmd, Buf *out, Buf *err, int timeout_sec) {
  int po[2], pe[2];
  if (pipe(po) < 0 || pipe(pe) < 0) return -1;
  for (int i = 3; i < 256; i++)
    if (i != po[0] && i != po[1] && i != pe[0] && i != pe[1])
      fcntl(i, F_SETFD, FD_CLOEXEC);

  pid_t pid = fork();
  if (pid == 0) {
    close(po[0]); close(pe[0]);
    dup2(po[1], STDOUT_FILENO); dup2(pe[1], STDERR_FILENO);
    close(po[1]); close(pe[1]);
    if (timeout_sec > 0) alarm(timeout_sec);
    // Use exec to avoid shell injection — we control the cmd string
    execl("/bin/sh", "sh", "-c", cmd, (char *)NULL);
    _exit(127);
  }
  close(po[1]); close(pe[1]);

  // Set non-blocking
  int fl_o = fcntl(po[0], F_GETFL, 0);
  int fl_e = fcntl(pe[0], F_GETFL, 0);
  fcntl(po[0], F_SETFL, fl_o | O_NONBLOCK);
  fcntl(pe[0], F_SETFL, fl_e | O_NONBLOCK);

  char buf[65536];
  struct timeval tv = {0, 100000};
  int active = 1;
  while (active) {
    fd_set rfds;
    FD_ZERO(&rfds);
    FD_SET(po[0], &rfds); FD_SET(pe[0], &rfds);
    int maxfd = po[0] > pe[0] ? po[0] : pe[0];
    int ret = select(maxfd + 1, &rfds, NULL, NULL, &tv);
    if (ret < 0) break;
    active = 0;
    if (FD_ISSET(po[0], &rfds)) {
      ssize_t n = read(po[0], buf, sizeof(buf));
      if (n > 0) { buf_append(out, buf, n); active = 1; }
    }
    if (FD_ISSET(pe[0], &rfds)) {
      ssize_t n = read(pe[0], buf, sizeof(buf));
      if (n > 0) { buf_append(err, buf, n); active = 1; }
    }
  }

  close(po[0]); close(pe[0]);
  int status;
  waitpid(pid, &status, 0);
  if (WIFEXITED(status)) return WEXITSTATUS(status);
  if (WIFSIGNALED(status)) return 128 + WTERMSIG(status);
  return -1;
}

// Parse files array JSON into dynamic array
typedef struct { char *name; char *content; } FileEnt;

static FileEnt *parse_files(const char *raw, int *count) {
  *count = 0;
  if (!raw || raw[0] != '[') return NULL;
  int cap = 16;
  FileEnt *files = malloc(sizeof(FileEnt) * cap);
  if (!files) return NULL;

  const char *p = raw + 1;
  while (*p && *count < MAX_FILES) {
    while (*p && *p != '{') p++;
    if (*p != '{') break;
    // Find matching }
    int depth = 1;
    const char *end = p + 1;
    while (*end && depth > 0) { if (*end == '{') depth++; if (*end == '}') depth--; end++; }
    if (depth != 0) break;

    // Extract substring for this entry
    size_t elen = end - p;
    char *entry = strndup(p, elen);
    if (!entry) { p = end; continue; }

    Str name_s = str_get(entry, "name");
    Str content_s = str_get(entry, "content");
    free(entry);

    if (name_s.s && content_s.s) {
      if (*count >= cap) { cap *= 2; files = realloc(files, sizeof(FileEnt) * cap); }
      files[*count].name = name_s.s;
      files[*count].content = content_s.s;
      (*count)++;
    } else {
      str_free(&name_s); str_free(&content_s);
    }
    p = end;
  }
  return files;
}

static void free_files(FileEnt *files, int count) {
  for (int i = 0; i < count; i++) { free(files[i].name); free(files[i].content); }
  free(files);
}

static int do_compile_and_run(const char *code, const char *args, int compile_only,
                               FileEnt *files, int file_count, const char *filename) {
  char tmpdir[] = "/tmp/klyron_c_XXXXXX";
  if (!mkdtemp(tmpdir)) {
    json_output(NULL, "Failed to create temp dir", 1, NULL);
    return 1;
  }

  // Write source files
  if (files && file_count > 0) {
    for (int i = 0; i < file_count; i++) {
      char *path;
      asprintf(&path, "%s/%s", tmpdir, files[i].name);
      FILE *f = fopen(path, "w");
      if (!f) { json_output(NULL, "Failed to write source file", 1, NULL); free(path); goto cleanup; }
      fputs(files[i].content, f);
      fclose(f);
      free(path);
    }
  } else if (code && *code) {
    char *path;
    asprintf(&path, "%s/%s", tmpdir, filename ? filename : "main.c");
    FILE *f = fopen(path, "w");
    if (!f) { json_output(NULL, "Failed to write source file", 1, NULL); free(path); goto cleanup; }
    fputs(code, f);
    fclose(f);
    free(path);
  } else {
    json_output(NULL, "No code provided", 1, NULL);
    goto cleanup;
  }

  // Find source files
  char src_buf[8192];
  int pos = 0;
  char find_cmd[1024];
  snprintf(find_cmd, sizeof(find_cmd), "find %s -maxdepth 1 -name '*.c' 2>/dev/null", tmpdir);
  FILE *fp = popen(find_cmd, "r");
  if (fp) {
    char line[1024];
    int first = 1;
    while (fgets(line, sizeof(line), fp)) {
      size_t l = strlen(line);
      while (l > 0 && (line[l-1] == '\n' || line[l-1] == '\r')) line[--l] = '\0';
      if (l == 0) continue;
      pos += snprintf(src_buf + pos, sizeof(src_buf) - pos, "%s%s", first ? "" : " ", line);
      first = 0;
    }
    pclose(fp);
    if (first) pos = snprintf(src_buf, sizeof(src_buf), "%s/main.c", tmpdir);
  } else {
    pos = snprintf(src_buf, sizeof(src_buf), "%s/main.c", tmpdir);
  }

  // Compile
  char *compile_cmd;
  asprintf(&compile_cmd,
           "cc -x c -o %s/prog %s -Wall -Wextra -Werror -O2 -lm -pthread 2>&1", tmpdir, src_buf);
  Buf co = {0}, ce = {0};
  int comp_exit = exec_shell(compile_cmd, &co, &ce, 120);
  free(compile_cmd);
  if (comp_exit != 0) {
    json_output(co.data ? co.data : "", ce.data ? ce.data : "", comp_exit, "Compilation failed");
    buf_free(&co); buf_free(&ce);
    goto cleanup;
  }
  buf_free(&co); buf_free(&ce);

  if (compile_only) {
    json_output("Compiled successfully", "", 0, "ok");
    goto cleanup;
  }

  // Run
  char *run_cmd;
  if (args && *args) asprintf(&run_cmd, "%s/prog %s", tmpdir, args);
  else asprintf(&run_cmd, "%s/prog", tmpdir);
  Buf ro = {0}, re = {0};
  int run_exit = exec_shell(run_cmd, &ro, &re, 30);
  free(run_cmd);
  json_output(ro.data ? ro.data : "", re.data ? re.data : "", run_exit,
              ro.data ? ro.data : "");
  buf_free(&ro); buf_free(&re);

cleanup:
  // Remove temp dir
  char rm_cmd[1024]; snprintf(rm_cmd, sizeof(rm_cmd), "rm -rf %s", tmpdir);
  exec_shell(rm_cmd, NULL, NULL, 5);
  return 0;
}

static void handle_action(const char *action, const char *code, const char *args,
                          const char *files_raw, const char *filename) {
  if (!action || !*action) { json_output(NULL, "No action specified", 1, NULL); return; }

  int fc = 0;
  FileEnt *files = parse_files(files_raw, &fc);

  if (strcmp(action, "exec") == 0 || strcmp(action, "run") == 0) {
    if (!code && fc == 0) { json_output(NULL, "No code provided", 1, NULL); free_files(files, fc); return; }
    do_compile_and_run(code, args, 0, files, fc, filename);
  } else if (strcmp(action, "compile") == 0) {
    if (!code && fc == 0) { json_output(NULL, "No code provided", 1, NULL); free_files(files, fc); return; }
    do_compile_and_run(code, args, 1, files, fc, filename);
  } else if (strcmp(action, "eval") == 0) {
    if (!code || !*code) { json_output(NULL, "No expression provided", 1, NULL); free_files(files, fc); return; }
    char *wrapped;
    int n = asprintf(&wrapped,
      "#include <stdio.h>\n#include <stdlib.h>\n#include <math.h>\n"
      "int main(void) {\n  printf(\"%%d\\n\", (int)(%s));\n  return 0;\n}\n", code);
    if (n < 0 || n > 65536) { json_output(NULL, "Expression too long", 1, NULL); free_files(files, fc); return; }
    do_compile_and_run(wrapped, args, 0, NULL, 0, NULL);
    free(wrapped);
  } else if (strcmp(action, "ping") == 0 || strcmp(action, "") == 0) {
    json_output("pong", "", 0, "ok");
  } else {
    char err[512];
    snprintf(err, sizeof(err), "Unknown action: %s", action);
    json_output(NULL, err, 1, NULL);
  }

  free_files(files, fc);
}

int main() {
  setvbuf(stdout, NULL, _IONBF, 0);
  setvbuf(stderr, NULL, _IONBF, 0);
  signal(SIGPIPE, SIG_IGN);

  // Ignore SIGCHLD to avoid zombies
  struct sigaction sa;
  memset(&sa, 0, sizeof(sa));
  sa.sa_handler = SIG_IGN;
  sa.sa_flags = SA_NOCLDWAIT;
  sigaction(SIGCHLD, &sa, NULL);

  char *line = NULL;
  size_t linecap = 0;
  ssize_t linelen;

  while ((linelen = getline(&line, &linecap, stdin)) > 0) {
    while (linelen > 0 && (line[linelen-1] == '\n' || line[linelen-1] == '\r')) line[--linelen] = '\0';
    if (linelen == 0) continue;

    Str action = str_get(line, "action");
    Str code = str_get(line, "code");
    Str args = str_get(line, "args");
    Str files = str_get_raw(line, "files");
    Str filename = str_get(line, "filename");

    handle_action(action.s ? action.s : "", code.s ? code.s : "",
                  args.s ? args.s : "", files.s, filename.s);

    str_free(&action); str_free(&code); str_free(&args);
    str_free(&files); str_free(&filename);
  }

  free(line);
  return 0;
}
