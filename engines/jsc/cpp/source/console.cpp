#include "klyron_jsc.h"
#include "cpp/impl/internal.h"
#include <cstdio>
#include <cstring>
#include <ctime>
#include <vector>

static void console_log_internal(klyron_jsc_engine_t* engine, const char* level, int argc, klyron_jsc_value_t** argv) {
    if (!engine || !engine->ctx) return;
    std::string out;
    out += "[";
    out += level;
    out += "] ";
    for (int i = 0; i < argc; i++) {
        if (i > 0) out += " ";
        if (!argv[i] || !argv[i]->value) {
            out += "undefined";
            continue;
        }
        JSValueRef exc = nullptr;
        JSStringRef str = JSValueToStringCopy(engine->ctx, argv[i]->value, &exc);
        if (str && !exc) {
            out += jsc_string_to_std(str);
            JSStringRelease(str);
        } else {
            out += "<error>";
        }
    }
    out += "\n";
    if (std::strcmp(level, "error") == 0 || std::strcmp(level, "warn") == 0) {
        std::fprintf(stderr, "%s", out.c_str());
    } else {
        std::fprintf(stdout, "%s", out.c_str());
    }
    std::fflush(nullptr);
}

klyron_jsc_value_t* klyron_jsc_console_log(klyron_jsc_engine_t* engine, int argc, klyron_jsc_value_t** argv) {
    console_log_internal(engine, "log", argc, argv);
    return nullptr;
}

klyron_jsc_value_t* klyron_jsc_console_warn(klyron_jsc_engine_t* engine, int argc, klyron_jsc_value_t** argv) {
    console_log_internal(engine, "warn", argc, argv);
    return nullptr;
}

klyron_jsc_value_t* klyron_jsc_console_error(klyron_jsc_engine_t* engine, int argc, klyron_jsc_value_t** argv) {
    console_log_internal(engine, "error", argc, argv);
    return nullptr;
}

klyron_jsc_value_t* klyron_jsc_console_info(klyron_jsc_engine_t* engine, int argc, klyron_jsc_value_t** argv) {
    console_log_internal(engine, "info", argc, argv);
    return nullptr;
}

klyron_jsc_value_t* klyron_jsc_console_debug(klyron_jsc_engine_t* engine, int argc, klyron_jsc_value_t** argv) {
    console_log_internal(engine, "debug", argc, argv);
    return nullptr;
}

klyron_jsc_value_t* klyron_jsc_console_table(klyron_jsc_engine_t* engine, int argc, klyron_jsc_value_t** argv) {
    if (!engine || !engine->ctx) return nullptr;
    if (argc < 1 || !argv[0]) {
        console_log_internal(engine, "table", 0, nullptr);
        return nullptr;
    }
    if (!JSValueIsObject(engine->ctx, argv[0]->value)) {
        console_log_internal(engine, "table", argc, argv);
        return nullptr;
    }
    JSObjectRef obj = (JSObjectRef)argv[0]->value;
    JSValueRef exc = nullptr;
    JSStringRef str = JSValueCreateJSONString(engine->ctx, obj, 2, &exc);
    if (str && !exc) {
        std::string s = jsc_string_to_std(str);
        JSStringRelease(str);
        std::fprintf(stdout, "[table] %s\n", s.c_str());
        std::fflush(stdout);
    } else {
        console_log_internal(engine, "table", argc, argv);
    }
    return nullptr;
}

klyron_jsc_value_t* klyron_jsc_console_assert(klyron_jsc_engine_t* engine, klyron_jsc_value_t* condition, int argc, klyron_jsc_value_t** argv) {
    if (!engine) return nullptr;
    bool cond = true;
    if (condition && engine->ctx) {
        cond = JSValueToBoolean(engine->ctx, condition->value);
    }
    if (!cond) {
        console_log_internal(engine, "AssertionError", argc, argv);
    }
    return nullptr;
}

klyron_jsc_value_t* klyron_jsc_console_count(klyron_jsc_engine_t* engine, const char* label) {
    if (!engine) return nullptr;
    static int counter = 0;
    const char* lbl = label ? label : "default";
    std::fprintf(stdout, "[count] %s: %d\n", lbl, ++counter);
    std::fflush(stdout);
    return nullptr;
}

klyron_jsc_value_t* klyron_jsc_console_time(klyron_jsc_engine_t* engine, const char* label) {
    if (!engine) return nullptr;
    const char* lbl = label ? label : "default";
    std::fprintf(stdout, "[time] %s: timer started\n", lbl);
    std::fflush(stdout);
    return nullptr;
}

klyron_jsc_value_t* klyron_jsc_console_time_end(klyron_jsc_engine_t* engine, const char* label) {
    if (!engine) return nullptr;
    const char* lbl = label ? label : "default";
    std::fprintf(stdout, "[timeEnd] %s: timer ended\n", lbl);
    std::fflush(stdout);
    return nullptr;
}

klyron_jsc_value_t* klyron_jsc_console_trace(klyron_jsc_engine_t* engine) {
    if (!engine) return nullptr;
    std::fprintf(stdout, "[trace]\n");
    std::fflush(stdout);
    return nullptr;
}

klyron_jsc_value_t* klyron_jsc_console_group(klyron_jsc_engine_t* engine, const char* label) {
    if (!engine) return nullptr;
    const char* lbl = label ? label : "group";
    std::fprintf(stdout, "[group] %s\n", lbl);
    std::fflush(stdout);
    return nullptr;
}

klyron_jsc_value_t* klyron_jsc_console_group_end(klyron_jsc_engine_t* engine) {
    if (!engine) return nullptr;
    std::fprintf(stdout, "[groupEnd]\n");
    std::fflush(stdout);
    return nullptr;
}
