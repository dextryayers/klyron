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
#include <limits.h>

#define MAX_OUTPUT (1 << 20)
#define MAX_LINE (1 << 18)
#define MAX_FILES 256

typedef struct { char *d; size_t len; size_t cap; } Buf;

static void b_init(Buf *b) { memset(b, 0, sizeof(*b)); }
static void b_free(Buf *b) { free(b->d); b->d = NULL; b->len = b->cap = 0; }
static int b_grow(Buf *b, size_t need) {
  if (b->len + need + 1 <= b->cap) return 0;
  size_t nc = b->cap ? b->cap : 4096;
  while (nc < b->len + need + 1) nc *= 2;
  char *p = realloc(b->d, nc);
  if (!p) return -1;
  b->d = p; b->cap = nc; return 0;
}
static int b_app(Buf *b, const char *s, size_t n) {
  if (b_grow(b, n) < 0) return -1;
  memcpy(b->d + b->len, s, n);
  b->len += n; b->d[b->len] = '\0';
  return 0;
}
static int b_appf(Buf *b, const char *fmt, ...) {
  va_list ap;
  va_start(ap, fmt); int n = vsnprintf(NULL, 0, fmt, ap); va_end(ap);
  if (n < 0) return -1;
  if (b_grow(b, n) < 0) return -1;
  va_start(ap, fmt); vsnprintf(b->d + b->len, n + 1, fmt, ap); va_end(ap);
  b->len += n;
  return 0;
}

static void json_esc(Buf *b, const char *s) {
  if (!s) return;
  for (const char *p = s; *p; p++) {
    unsigned char c = *p;
    switch (c) {
      case '"': b_app(b, "\\\"", 2); break;
      case '\\': b_app(b, "\\\\", 2); break;
      case '\n': b_app(b, "\\n", 2); break;
      case '\r': b_app(b, "\\r", 2); break;
      case '\t': b_app(b, "\\t", 2); break;
      default:
        if (c < 0x20) b_appf(b, "\\u%04x", c);
        else b_app(b, (const char*)&c, 1);
    }
  }
}

static void json_out_ex(const char *so, const char *se, int ec, const char *res, const char *dj) {
  Buf b; b_init(&b);
  b_app(&b, "{\"stdout\":\"", 11); json_esc(&b, so);
  b_app(&b, "\",\"stderr\":\"", 12); json_esc(&b, se);
  b_appf(&b, "\",\"exit_code\":%d,\"result\":\"", ec); json_esc(&b, res);
  b_app(&b, "\"", 1);
  if (dj) { b_app(&b, ",\"diagnostics\":", 16); b_app(&b, dj, strlen(dj)); }
  b_app(&b, "}\n", 2);
  write(STDOUT_FILENO, b.d, b.len);
  b_free(&b);
}

static void json_out(const char *so, const char *se, int ec, const char *res) {
  json_out_ex(so, se, ec, res, NULL);
}

typedef struct { char *s; } Str;
static void str_free(Str *r) { free(r->s); r->s = NULL; }

static Str str_get(const char *json, const char *key) {
  Str r = {NULL};
  size_t kl = strlen(key);
  const char *p = json;
  while ((p = strstr(p, key))) {
    if ((p == json || p[-1] == '"') && p[kl] == '"') {
      p += kl; while (*p && *p != ':') p++; if (*p != ':') { p++; continue; }
      p++; while (*p == ' ' || *p == '\t' || *p == '\n' || *p == '\r') p++;
      if (*p != '"') { p++; continue; }
      p++;
      Buf b; b_init(&b);
      while (*p && *p != '"') {
        if (*p == '\\') {
          p++;
          switch (*p) {
            case '"': b_app(&b, "\"", 1); break;
            case '\\': b_app(&b, "\\", 1); break;
            case '/': b_app(&b, "/", 1); break;
            case 'n': b_app(&b, "\n", 1); break;
            case 't': b_app(&b, "\t", 1); break;
            case 'r': b_app(&b, "\r", 1); break;
            case 'b': b_app(&b, "\b", 1); break;
            case 'f': b_app(&b, "\f", 1); break;
            case 'u': {
              p++; char hex[5] = {p[0], p[1], p[2], p[3], 0};
              if (strspn(hex, "0123456789abcdefABCDEF") == 4) {
                unsigned long cp = strtoul(hex, NULL, 16);
                if (cp < 0x80) { char c2 = cp; b_app(&b, &c2, 1); }
                else if (cp < 0x800) { char c2 = 0xC0 | (cp >> 6), c3 = 0x80 | (cp & 0x3F); b_app(&b, &c2, 1); b_app(&b, &c3, 1); }
                else { char c2 = 0xE0 | (cp >> 12), c3 = 0x80 | ((cp >> 6) & 0x3F), c4 = 0x80 | (cp & 0x3F); b_app(&b, &c2, 1); b_app(&b, &c3, 1); b_app(&b, &c4, 1); }
              }
              p += 4; break;
            }
            default: if (*p) b_app(&b, p, 1); break;
          }
          p++;
          continue;
        }
        b_app(&b, p, 1); p++;
      }
      r.s = b.d; return r;
    }
    p++;
  }
  return r;
}

static Str str_get_raw(const char *json, const char *key) {
  Str r = {NULL};
  size_t kl = strlen(key);
  const char *p = json;
  while ((p = strstr(p, key))) {
    if ((p == json || p[-1] == '"') && p[kl] == '"') {
      p += kl; while (*p && *p != ':') p++; if (*p != ':') { p++; continue; }
      p++; while (*p == ' ' || *p == '\t' || *p == '\n' || *p == '\r') p++;
      if (*p == '"') {
        const char *start = p; p++;
        while (*p) { if (*p == '"') break; if (*p == '\\') p++; p++; }
        if (*p == '"') p++;
        r.s = strndup(start, p - start); return r;
      }
      if (*p == '[' || *p == '{') {
        int depth = 1; const char *start = p; p++;
        while (*p && depth > 0) {
          if (*p == '{' || *p == '[') depth++;
          if (*p == '}' || *p == ']') depth--;
          if (*p == '"') { p++; while (*p && !(*p == '"' && *(p-1) != '\\')) p++; if (*p) p++; }
          else p++;
        }
        r.s = strndup(start, p - start); return r;
      }
      { const char *s = p; while (*p && *p != ',' && *p != '}' && *p != ']' && *p != '\n') p++; r.s = strndup(s, p - s); return r; }
    }
    p++;
  }
  return r;
}

static int exec_sh(const char *cmd, Buf *out, Buf *err, int tmo) {
  int po[2], pe[2];
  if (pipe(po) < 0 || pipe(pe) < 0) return -1;
  for (int i = 3; i < 256; i++)
    if (i != po[0] && i != po[1] && i != pe[0] && i != pe[1])
      fcntl(i, F_SETFD, FD_CLOEXEC);

  // Temporarily disable SA_NOCLDWAIT so waitpid can collect exit status
  struct sigaction old_sa;
  memset(&old_sa, 0, sizeof(old_sa));
  struct sigaction dfl_sa; memset(&dfl_sa, 0, sizeof(dfl_sa)); dfl_sa.sa_handler = SIG_DFL;
  sigaction(SIGCHLD, &dfl_sa, &old_sa);

  pid_t pid = fork();
  if (pid == 0) {
    // Child: reset signals to default (already done above, but be explicit)
    struct sigaction sa; memset(&sa, 0, sizeof(sa)); sa.sa_handler = SIG_DFL;
    sigaction(SIGPIPE, &sa, NULL);
    close(po[0]); close(pe[0]);
    dup2(po[1], STDOUT_FILENO); dup2(pe[1], STDERR_FILENO);
    close(po[1]); close(pe[1]);
    if (tmo > 0) alarm(tmo);
    execl("/bin/sh", "sh", "-c", cmd, (char *)NULL);
    _exit(127);
  }
  close(po[1]); close(pe[1]);

  int fl_o = fcntl(po[0], F_GETFL, 0);
  int fl_e = fcntl(pe[0], F_GETFL, 0);
  fcntl(po[0], F_SETFL, fl_o | O_NONBLOCK);
  fcntl(pe[0], F_SETFL, fl_e | O_NONBLOCK);

  char buf[65536];
  int active = 1;
  while (active) {
    fd_set rfds; FD_ZERO(&rfds);
    FD_SET(po[0], &rfds); FD_SET(pe[0], &rfds);
    int mf = po[0] > pe[0] ? po[0] : pe[0];
    struct timeval tv = {0, 100000};
    int ret;
    do { ret = select(mf + 1, &rfds, NULL, NULL, &tv); } while (ret < 0 && errno == EINTR);
    if (ret < 0) break;
    active = 0;
    if (FD_ISSET(po[0], &rfds)) { ssize_t n = read(po[0], buf, sizeof(buf)); if (n > 0) { if (out) b_app(out, buf, n); active = 1; } }
    if (FD_ISSET(pe[0], &rfds)) { ssize_t n = read(pe[0], buf, sizeof(buf)); if (n > 0) { if (err) b_app(err, buf, n); active = 1; } }
  }

  close(po[0]); close(pe[0]);
  int status = -1;
  int wret;
  do { wret = waitpid(pid, &status, 0); } while (wret < 0 && errno == EINTR);

  // Restore SA_NOCLDWAIT
  sigaction(SIGCHLD, &old_sa, NULL);

  if (wret < 0) return -1;
  if (WIFEXITED(status)) return WEXITSTATUS(status);
  return 128 + (WIFSIGNALED(status) ? WTERMSIG(status) : 0);
}

typedef struct { char *name; char *content; } FileEnt;

static FileEnt *parse_files(const char *raw, int *count) {
  *count = 0;
  if (!raw || *raw != '[') return NULL;
  int cap = 16;
  FileEnt *files = malloc(sizeof(FileEnt) * cap);
  if (!files) return NULL;

  const char *p = raw + 1;
  while (*p && *count < MAX_FILES) {
    while (*p && *p != '{') p++;
    if (*p != '{') break;
    int depth = 1; const char *end = p + 1;
    while (*end && depth > 0) { if (*end == '{') depth++; if (*end == '}') depth--; end++; }
    if (depth != 0) break;

    char *entry = strndup(p, end - p);
    if (!entry) { p = end; continue; }
    Str ns = str_get(entry, "name");
    Str cs = str_get(entry, "content");
    free(entry);
    if (ns.s && cs.s) {
      if (*count >= cap) {
        cap *= 2;
        FileEnt *nf = realloc(files, sizeof(FileEnt) * cap);
        if (!nf) { free(ns.s); free(cs.s); break; }
        files = nf;
      }
      files[*count].name = ns.s;
      files[*count].content = cs.s;
      (*count)++;
    } else { str_free(&ns); str_free(&cs); }
    p = end;
  }
  return files;
}

static void free_files(FileEnt *f, int n) {
  for (int i = 0; i < n; i++) { free(f[i].name); free(f[i].content); }
  free(f);
}

static int do_cr(const char *code, const char *args, int co, FileEnt *files, int fc, const char *fn) {
  char tmpdir[] = "/tmp/klyron_c_XXXXXX";
  if (!mkdtemp(tmpdir)) { json_out(NULL, "Failed to create temp dir", 1, NULL); return 1; }

  int wrote = 0;
  if (files && fc > 0) {
    for (int i = 0; i < fc; i++) {
      char *p; if (asprintf(&p, "%s/%s", tmpdir, files[i].name) < 0) continue;
      FILE *f = fopen(p, "w"); free(p);
      if (!f) { json_out(NULL, "Failed to write source", 1, NULL); goto cleanup; }
      fputs(files[i].content, f); fclose(f); wrote = 1;
    }
  }
  if (!wrote && code && *code) {
    char *p; asprintf(&p, "%s/%s", tmpdir, fn ? fn : "main.c");
    FILE *f = fopen(p, "w"); free(p);
    if (!f) { json_out(NULL, "Failed to write source", 1, NULL); goto cleanup; }
    fputs(code, f); fclose(f);
  }
  if (!wrote && (!code || !*code)) { json_out(NULL, "No code", 1, NULL); goto cleanup; }

  // Collect source files via find
  Buf sb; b_init(&sb);
  char fc_c[1024]; snprintf(fc_c, sizeof(fc_c), "find %s -maxdepth 1 -name '*.c' 2>/dev/null", tmpdir);
  FILE *fp = popen(fc_c, "r");
  int found = 0;
  if (fp) {
    char ln[4096];
    while (fgets(ln, sizeof(ln), fp)) {
      size_t l = strlen(ln); while (l > 0 && (ln[l-1] == '\n' || ln[l-1] == '\r')) ln[--l] = '\0';
      if (!l) continue;
      if (found) b_app(&sb, " ", 1);
      b_app(&sb, ln, l); found = 1;
    }
    pclose(fp);
  }
  if (!found) b_app(&sb, tmpdir, strlen(tmpdir)), b_app(&sb, "/main.c", 7);

  // Compile
  char *cc; asprintf(&cc, "cc -x c -o %s/prog %s -Wall -Wextra -Werror -O2 -lm -pthread 2>&1", tmpdir, sb.d);
  b_free(&sb);
  Buf co2 = {0}, ce2 = {0};
  int ce = exec_sh(cc, &co2, &ce2, 120); free(cc);
  if (ce != 0) { json_out(co2.d ? co2.d : "", ce2.d ? ce2.d : "", ce, "Compilation failed"); b_free(&co2); b_free(&ce2); goto cleanup; }
  b_free(&co2); b_free(&ce2);
  if (co) { json_out("Compiled successfully", "", 0, "ok"); goto cleanup; }

  // Run
  char *rc; if (args && *args) asprintf(&rc, "%s/prog %s", tmpdir, args); else asprintf(&rc, "%s/prog", tmpdir);
  Buf ro = {0}, re2 = {0};
  int re = exec_sh(rc, &ro, &re2, 30); free(rc);
  json_out(ro.d ? ro.d : "", re2.d ? re2.d : "", re, ro.d ? ro.d : "");
  b_free(&ro); b_free(&re2);

cleanup:
  { char rm[1024]; snprintf(rm, sizeof(rm), "rm -rf %s", tmpdir); exec_sh(rm, NULL, NULL, 5); }
  return 0;
}

static void handle_action(const char *action, const char *code, const char *args,
                          const char *files_raw, const char *filename) {
  if (!action || !*action) { json_out(NULL, "No action specified", 1, NULL); return; }
  int fc = 0;
  FileEnt *files = parse_files(files_raw, &fc);

  if (strcmp(action, "exec") == 0 || strcmp(action, "run") == 0) {
    if (!code && fc == 0) { json_out(NULL, "No code provided", 1, NULL); free_files(files, fc); return; }
    do_cr(code, args, 0, files, fc, filename);
  } else if (strcmp(action, "compile") == 0) {
    if (!code && fc == 0) { json_out(NULL, "No code provided", 1, NULL); free_files(files, fc); return; }
    do_cr(code, args, 1, files, fc, filename);
  } else if (strcmp(action, "eval") == 0) {
    if (!code || !*code) { json_out(NULL, "No expression", 1, NULL); free_files(files, fc); return; }
    char *w; asprintf(&w, "#include <stdio.h>\n#include <stdlib.h>\n#include <math.h>\nint main(void) { printf(\"%%d\\n\", (int)(%s)); return 0; }\n", code);
    if (!w) { json_out(NULL, "Allocation failed", 1, NULL); free_files(files, fc); return; }
    do_cr(w, args, 0, NULL, 0, NULL); free(w);
  } else if (strcmp(action, "ping") == 0 || strcmp(action, "") == 0) {
    json_out("pong", "", 0, "ok");
  } else {
    json_out(NULL, "Unknown action", 1, NULL);
  }
  free_files(files, fc);
}

int main() {
  setvbuf(stdout, NULL, _IONBF, 0);
  setvbuf(stderr, NULL, _IONBF, 0);
  signal(SIGPIPE, SIG_IGN);

  struct sigaction sa; memset(&sa, 0, sizeof(sa));
  sa.sa_handler = SIG_IGN; sa.sa_flags = SA_NOCLDWAIT;
  sigaction(SIGCHLD, &sa, NULL);

  char *line = NULL; size_t linecap = 0;
  while (getline(&line, &linecap, stdin) > 0) {
    size_t l = strlen(line);
    while (l > 0 && (line[l-1] == '\n' || line[l-1] == '\r')) line[--l] = '\0';
    if (!l) continue;
    Str a = str_get(line, "action"), c = str_get(line, "code");
    Str ar = str_get(line, "args"), f = str_get_raw(line, "files");
    Str fn = str_get(line, "filename");
    handle_action(a.s ? a.s : "", c.s ? c.s : "", ar.s ? ar.s : "", f.s, fn.s);
    str_free(&a); str_free(&c); str_free(&ar); str_free(&f); str_free(&fn);
  }
  free(line);
  return 0;
}
