#ifndef KLYRON_JSC_INTERNAL_H
#define KLYRON_JSC_INTERNAL_H

#include "klyron_jsc.h"
#include <JavaScriptCore/JavaScript.h>
#include <cstring>
#include <cstdlib>
#include <string>
#include <memory>
#include <mutex>
#include <atomic>

/* ─── Internal struct definitions ─────────────────────────────── */

struct klyron_jsc_engine {
    JSContextGroupRef group;
    JSGlobalContextRef ctx;
    char error_buf[KLYRON_JSC_ERROR_BUF_SIZE];

    klyron_jsc_engine()
        : group(nullptr), ctx(nullptr) { error_buf[0] = '\0'; }

    ~klyron_jsc_engine() {
        if (ctx) { JSGlobalContextRelease(ctx); ctx = nullptr; }
        if (group) { JSContextGroupRelease(group); group = nullptr; }
    }
};

struct klyron_jsc_script {
    JSStringRef source;
    JSStringRef filename;
    klyron_jsc_engine_t* parent;

    klyron_jsc_script(JSStringRef src, JSStringRef fn, klyron_jsc_engine_t* p)
        : source(src), filename(fn), parent(p) {}

    ~klyron_jsc_script() {
        if (source)   JSStringRelease(source);
        if (filename) JSStringRelease(filename);
    }
};

struct klyron_jsc_value {
    JSGlobalContextRef ctx;
    JSValueRef value;
    bool is_protected;

    klyron_jsc_value(JSGlobalContextRef c, JSValueRef v)
        : ctx(c), value(v), is_protected(false) {}

    ~klyron_jsc_value() {
        if (is_protected && ctx && value) {
            JSValueUnprotect(ctx, value);
        }
    }

    void protect() {
        if (!is_protected && ctx && value) {
            JSValueProtect(ctx, value);
            is_protected = true;
        }
    }
};

/* ─── Helper functions ────────────────────────────────────────── */

inline std::string jsc_string_to_std(JSStringRef str) {
    if (!str) return {};
    size_t max_size = JSStringGetMaximumUTF8CStringSize(str);
    if (max_size == 0) return {};
    std::string result(max_size - 1, '\0');
    size_t written = JSStringGetUTF8CString(str, &result[0], max_size);
    result.resize(written);
    return result;
}

inline JSStringRef jsc_string_from_cstr(const char* s) {
    return s ? JSStringCreateWithUTF8CString(s) : nullptr;
}

inline std::string jsc_val_to_std(JSContextRef ctx, JSValueRef val, JSValueRef* exc) {
    if (!val) return {};
    JSStringRef str = JSValueToStringCopy(ctx, val, exc);
    if (!str) return {};
    std::string result = jsc_string_to_std(str);
    JSStringRelease(str);
    return result;
}

inline void jsc_set_error(klyron_jsc_engine_t* engine, const std::string& msg) {
    if (engine) {
        std::strncpy(engine->error_buf, msg.c_str(), KLYRON_JSC_ERROR_BUF_SIZE - 1);
        engine->error_buf[KLYRON_JSC_ERROR_BUF_SIZE - 1] = '\0';
    }
}

inline void jsc_set_string_result(klyron_jsc_string_result_t* result, const std::string& s) {
    char* buf = (char*)std::malloc(s.size() + 1);
    if (buf) {
        std::memcpy(buf, s.c_str(), s.size() + 1);
        result->data = buf;
        result->length = s.size();
    }
    result->success = true;
}

inline void jsc_set_bool_result(klyron_jsc_result_t* result, bool ok) {
    result->success = ok;
    if (!ok) {
        std::strncpy(result->error, "operation failed", KLYRON_JSC_ERROR_BUF_SIZE - 1);
    }
}

inline void jsc_capture_exception(klyron_jsc_engine_t* engine, JSValueRef exc) {
    if (!exc || !engine) return;
    std::string msg = jsc_val_to_std(engine->ctx, exc, nullptr);
    if (!msg.empty()) {
        jsc_set_error(engine, msg);
    }
}

inline std::string jsc_error_to_string(klyron_jsc_engine_t* engine) {
    return engine ? std::string(engine->error_buf) : std::string("null engine");
}

#endif /* KLYRON_JSC_INTERNAL_H */
