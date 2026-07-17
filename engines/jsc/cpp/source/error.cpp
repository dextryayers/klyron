#include "klyron_jsc.h"
#include "cpp/impl/internal.h"

const char* klyron_jsc_get_exception_message(klyron_jsc_engine_t* engine) {
    if (!engine) return "Null engine";
    return engine->error_buf;
}

klyron_jsc_string_result_t klyron_jsc_get_stack_trace(klyron_jsc_engine_t* engine) {
    klyron_jsc_string_result_t result = {false, nullptr, 0, {0}};
    if (!engine || !engine->ctx) return result;

    const char* code =
        "try { throw new Error(); } catch(e) { return e.stack || ''; }";

    JSStringRef src = jsc_string_from_cstr(code);
    if (!src) return result;

    JSValueRef exc = nullptr;
    JSValueRef val = JSEvaluateScript(engine->ctx, src, nullptr,
                                       jsc_string_from_cstr("<stack>"), 1, &exc);
    JSStringRelease(src);

    if (exc) {
        jsc_capture_exception(engine, exc);
        return result;
    }

    std::string s = jsc_val_to_std(engine->ctx, val ? val : JSValueMakeUndefined(engine->ctx), &exc);
    if (exc) {
        jsc_capture_exception(engine, exc);
        return result;
    }
    jsc_set_string_result(&result, s);
    return result;
}
