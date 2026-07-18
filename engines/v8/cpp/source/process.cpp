#include "klyron_v8.h"
#include "cpp/impl/internal.h"

#include <cstring>
#include <cstdlib>
#include <string>
#include <unistd.h>
#include <climits>

#ifdef __linux__
#include <sys/prctl.h>
#endif

klyron_v8_process_info_t* klyron_v8_process_info(klyron_v8_context_t* ctx) {
    auto* info = static_cast<klyron_v8_process_info_t*>(std::malloc(sizeof(klyron_v8_process_info_t)));
    if (!info) return nullptr;
    std::memset(info, 0, sizeof(*info));

    info->pid = (uint64_t)getpid();
    info->ppid = (uint64_t)getppid();

    char cwd_buf[PATH_MAX];
    if (getcwd(cwd_buf, sizeof(cwd_buf))) {
        info->cwd = strdup(cwd_buf);
    }

#ifdef __linux__
    char exe_buf[PATH_MAX];
    ssize_t exe_len = readlink("/proc/self/exe", exe_buf, sizeof(exe_buf) - 1);
    if (exe_len > 0) {
        exe_buf[exe_len] = '\0';
        info->exec_path = strdup(exe_buf);
    }
#endif

    info->platform = strdup("linux");

    const char* title = "";
#ifdef __linux__
    char tbuf[256];
    if (prctl(PR_GET_NAME, tbuf, 0, 0, 0) == 0) {
        title = tbuf;
    }
#endif
    info->title = strdup(title);

    extern char** environ;
    int count = 0;
    while (environ[count]) count++;
    info->argc = count;
    if (count > 0) {
        info->argv = static_cast<char**>(std::malloc(sizeof(char*) * (count + 1)));
        for (int i = 0; i < count; i++) {
            info->argv[i] = strdup(environ[i]);
        }
        info->argv[count] = nullptr;
    }

    return info;
}

klyron_v8_result_t klyron_v8_process_exit(klyron_v8_context_t* ctx, int code) {
    klyron_v8_result_t result = {false, {0}};
    std::exit(code);
    return result;
}

klyron_v8_string_result_t klyron_v8_process_env_get(klyron_v8_context_t* ctx, const char* name) {
    klyron_v8_string_result_t result = {false, nullptr, 0, {0}};
    if (!name) return result;

    const char* val = std::getenv(name);
    if (val) {
        set_result(&result, std::string(val));
    }
    return result;
}

klyron_v8_value_t* klyron_v8_process_env_all(klyron_v8_context_t* ctx) {
    if (!ctx) return nullptr;

    auto iso = get_iso(ctx);
    v8::Locker locker(iso);
    v8::Isolate::Scope iso_scope(iso);
    v8::HandleScope scope(iso);
    auto context = get_ctx(ctx);
    v8::Context::Scope ctx_scope(context);

    auto obj = v8::Object::New(iso);

    extern char** environ;
    for (int i = 0; environ[i]; i++) {
        std::string entry(environ[i]);
        size_t eq = entry.find('=');
        if (eq != std::string::npos) {
            auto key = v8::String::NewFromUtf8(iso, entry.substr(0, eq).c_str(), v8::NewStringType::kNormal).ToLocalChecked();
            auto val = v8::String::NewFromUtf8(iso, entry.substr(eq + 1).c_str(), v8::NewStringType::kNormal).ToLocalChecked();
            obj->Set(context, key, val).Check();
        }
    }

    return new klyron_v8_value(iso, obj, ctx->parent);
}

void klyron_v8_process_info_dispose(klyron_v8_process_info_t* info) {
    if (!info) return;
    std::free(info->exec_path);
    if (info->argv) {
        for (int i = 0; i < info->argc; i++) {
            std::free(info->argv[i]);
        }
        std::free(info->argv);
    }
    std::free(info->cwd);
    std::free(info->platform);
    std::free(info->title);
    std::free(info);
}
