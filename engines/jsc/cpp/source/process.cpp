#include "klyron_jsc.h"
#include "cpp/impl/internal.h"
#include <cstring>
#include <cstdlib>
#include <unistd.h>
#include <climits>
#include <pwd.h>

extern char** environ;

klyron_jsc_value_t* klyron_jsc_process_pid(klyron_jsc_engine_t* engine) {
    if (!engine) return nullptr;
    auto v = new klyron_jsc_value_t(engine->ctx, JSValueMakeNumber(engine->ctx, (double)getpid()));
    v->protect();
    return v;
}

klyron_jsc_value_t* klyron_jsc_process_ppid(klyron_jsc_engine_t* engine) {
    if (!engine) return nullptr;
    auto v = new klyron_jsc_value_t(engine->ctx, JSValueMakeNumber(engine->ctx, (double)getppid()));
    v->protect();
    return v;
}

klyron_jsc_value_t* klyron_jsc_process_cwd(klyron_jsc_engine_t* engine) {
    if (!engine) return nullptr;
    char buf[PATH_MAX];
    if (!getcwd(buf, sizeof(buf))) {
        jsc_set_error(engine, "process.cwd: failed to get current directory");
        return nullptr;
    }
    JSStringRef jsstr = jsc_string_from_cstr(buf);
    JSValueRef val = JSValueMakeString(engine->ctx, jsstr);
    JSStringRelease(jsstr);
    auto v = new klyron_jsc_value_t(engine->ctx, val);
    v->protect();
    return v;
}

klyron_jsc_string_result_t klyron_jsc_process_cwd_str(klyron_jsc_engine_t* engine) {
    klyron_jsc_string_result_t result = {false, nullptr, 0, {0}};
    if (!engine) return result;
    char buf[PATH_MAX];
    if (!getcwd(buf, sizeof(buf))) return result;
    jsc_set_string_result(&result, std::string(buf));
    return result;
}

klyron_jsc_string_result_t klyron_jsc_process_exec_path(klyron_jsc_engine_t* engine) {
    klyron_jsc_string_result_t result = {false, nullptr, 0, {0}};
    if (!engine) return result;
    char buf[PATH_MAX];
    ssize_t len = readlink("/proc/self/exe", buf, sizeof(buf) - 1);
    if (len > 0) {
        buf[len] = '\0';
        jsc_set_string_result(&result, std::string(buf));
    }
    return result;
}

klyron_jsc_value_t* klyron_jsc_process_platform(klyron_jsc_engine_t* engine) {
    if (!engine) return nullptr;
    const char* platform =
#ifdef _WIN32
        "win32";
#elif defined(__APPLE__)
        "darwin";
#elif defined(__linux__)
        "linux";
#elif defined(__FreeBSD__)
        "freebsd";
#else
        "unknown";
#endif
    JSStringRef jsstr = jsc_string_from_cstr(platform);
    JSValueRef val = JSValueMakeString(engine->ctx, jsstr);
    JSStringRelease(jsstr);
    auto v = new klyron_jsc_value_t(engine->ctx, val);
    v->protect();
    return v;
}

klyron_jsc_value_t* klyron_jsc_process_arch(klyron_jsc_engine_t* engine) {
    if (!engine) return nullptr;
    const char* arch =
#if defined(__x86_64__) || defined(_M_X64)
        "x64";
#elif defined(__i386__) || defined(_M_IX86)
        "ia32";
#elif defined(__aarch64__) || defined(_M_ARM64)
        "arm64";
#elif defined(__arm__) || defined(_M_ARM)
        "arm";
#else
        "unknown";
#endif
    JSStringRef jsstr = jsc_string_from_cstr(arch);
    JSValueRef val = JSValueMakeString(engine->ctx, jsstr);
    JSStringRelease(jsstr);
    auto v = new klyron_jsc_value_t(engine->ctx, val);
    v->protect();
    return v;
}

klyron_jsc_value_t* klyron_jsc_process_env_get(klyron_jsc_engine_t* engine, const char* name) {
    if (!engine || !name) return nullptr;
    const char* val = std::getenv(name);
    if (!val) {
        auto v = new klyron_jsc_value_t(engine->ctx, JSValueMakeUndefined(engine->ctx));
        v->protect();
        return v;
    }
    JSStringRef jsstr = jsc_string_from_cstr(val);
    JSValueRef jsval = JSValueMakeString(engine->ctx, jsstr);
    JSStringRelease(jsstr);
    auto v = new klyron_jsc_value_t(engine->ctx, jsval);
    v->protect();
    return v;
}

klyron_jsc_value_t* klyron_jsc_process_env_all(klyron_jsc_engine_t* engine) {
    if (!engine) return nullptr;
    JSObjectRef obj = JSObjectMake(engine->ctx, nullptr, nullptr);
    JSValueRef exc = nullptr;
    for (char** env = environ; *env; env++) {
        std::string entry(*env);
        size_t eq = entry.find('=');
        if (eq == std::string::npos) continue;
        std::string key = entry.substr(0, eq);
        std::string val = entry.substr(eq + 1);
        JSStringRef key_str = jsc_string_from_cstr(key.c_str());
        JSStringRef val_str = jsc_string_from_cstr(val.c_str());
        JSValueRef js_val = JSValueMakeString(engine->ctx, val_str);
        JSObjectSetProperty(engine->ctx, obj, key_str, js_val, kJSPropertyAttributeNone, &exc);
        JSStringRelease(val_str);
        JSStringRelease(key_str);
    }
    auto v = new klyron_jsc_value_t(engine->ctx, obj);
    v->protect();
    return v;
}

klyron_jsc_value_t* klyron_jsc_process_argv(klyron_jsc_engine_t* engine) {
    if (!engine) return nullptr;
    JSValueRef exc = nullptr;
    JSObjectRef arr = JSObjectMakeArray(engine->ctx, 0, nullptr, &exc);
    if (exc) return nullptr;
    int idx = 0;
    char buf[256];
    std::snprintf(buf, sizeof(buf), "klyron");
    JSStringRef val_str = jsc_string_from_cstr(buf);
    JSValueRef val = JSValueMakeString(engine->ctx, val_str);
    JSStringRef idx_str = JSStringCreateWithUTF8CString("0");
    JSObjectSetProperty(engine->ctx, arr, idx_str, val, kJSPropertyAttributeNone, &exc);
    JSStringRelease(idx_str);
    JSStringRelease(val_str);
    auto v = new klyron_jsc_value_t(engine->ctx, arr);
    v->protect();
    return v;
}

klyron_jsc_value_t* klyron_jsc_process_exit(klyron_jsc_engine_t* engine, int code) {
    if (!engine) return nullptr;
    std::exit(code);
    return nullptr;
}

klyron_jsc_value_t* klyron_jsc_process_uptime(klyron_jsc_engine_t* engine) {
    if (!engine) return nullptr;
    auto v = new klyron_jsc_value_t(engine->ctx, JSValueMakeNumber(engine->ctx, 0.0));
    v->protect();
    return v;
}

klyron_jsc_value_t* klyron_jsc_process_memory_usage(klyron_jsc_engine_t* engine) {
    if (!engine) return nullptr;
    JSObjectRef obj = JSObjectMake(engine->ctx, nullptr, nullptr);
    auto set_num = [&](const char* name, double val) {
        JSStringRef key = jsc_string_from_cstr(name);
        JSObjectSetProperty(engine->ctx, obj, key, JSValueMakeNumber(engine->ctx, val), kJSPropertyAttributeNone, nullptr);
        JSStringRelease(key);
    };
    set_num("rss", 0.0);
    set_num("heapTotal", 0.0);
    set_num("heapUsed", 0.0);
    set_num("external", 0.0);
    set_num("arrayBuffers", 0.0);
    auto v = new klyron_jsc_value_t(engine->ctx, obj);
    v->protect();
    return v;
}

klyron_jsc_string_result_t klyron_jsc_process_title(klyron_jsc_engine_t* engine) {
    klyron_jsc_string_result_t result = {false, nullptr, 0, {0}};
    if (!engine) return result;
    jsc_set_string_result(&result, std::string("klyron"));
    return result;
}
