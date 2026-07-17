#include "klyron_jsc.h"
#include "cpp/impl/internal.h"

klyron_jsc_script_t* klyron_jsc_compile(klyron_jsc_engine_t* engine,
                                         const char* source,
                                         const char* filename) {
    if (!engine || !engine->ctx || !source) return nullptr;

    JSStringRef src = jsc_string_from_cstr(source);
    if (!src) return nullptr;
    JSStringRef fn = filename ? jsc_string_from_cstr(filename) : jsc_string_from_cstr("<eval>");
    if (!fn) {
        JSStringRelease(src);
        return nullptr;
    }

    return new klyron_jsc_script_t(src, fn, engine);
}

klyron_jsc_string_result_t klyron_jsc_run(klyron_jsc_engine_t* engine,
                                           klyron_jsc_script_t* script) {
    klyron_jsc_string_result_t result = {false, nullptr, 0, {0}};
    if (!engine || !script || !engine->ctx) {
        jsc_set_string_result(&result, "");
        return result;
    }

    JSValueRef exc = nullptr;
    JSValueRef val = JSEvaluateScript(engine->ctx, script->source,
                                       nullptr, script->filename, 1, &exc);
    if (exc) {
        jsc_capture_exception(engine, exc);
        return result;
    }

    std::string str = jsc_val_to_std(engine->ctx, val ? val : JSValueMakeUndefined(engine->ctx), &exc);
    if (exc) {
        jsc_capture_exception(engine, exc);
        return result;
    }
    jsc_set_string_result(&result, str);
    return result;
}

klyron_jsc_string_result_t klyron_jsc_eval(klyron_jsc_engine_t* engine,
                                            const char* source,
                                            const char* filename) {
    klyron_jsc_string_result_t result = {false, nullptr, 0, {0}};
    if (!engine || !engine->ctx || !source) return result;

    JSStringRef src = jsc_string_from_cstr(source);
    JSStringRef fn = filename ? jsc_string_from_cstr(filename) : jsc_string_from_cstr("<eval>");
    if (!src || !fn) {
        if (src) JSStringRelease(src);
        if (fn) JSStringRelease(fn);
        jsc_set_error(engine, "failed to create JSString");
        return result;
    }

    JSValueRef exc = nullptr;
    JSValueRef val = JSEvaluateScript(engine->ctx, src, nullptr, fn, 1, &exc);

    JSStringRelease(src);
    JSStringRelease(fn);

    if (exc) {
        jsc_capture_exception(engine, exc);
        return result;
    }

    std::string str = jsc_val_to_std(engine->ctx, val ? val : JSValueMakeUndefined(engine->ctx), &exc);
    if (exc) {
        jsc_capture_exception(engine, exc);
        return result;
    }
    jsc_set_string_result(&result, str);
    return result;
}

void klyron_jsc_script_dispose(klyron_jsc_script_t* script) {
    delete script;
}
