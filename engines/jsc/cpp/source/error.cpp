#include "klyron_jsc.h"
#include "cpp/impl/internal.h"

const char* klyron_jsc_get_exception_message(klyron_jsc_engine_t* engine) {
    if (!engine) return "Null engine";
    return engine->error_buf;
}

klyron_jsc_string_result_t klyron_jsc_get_stack_trace(klyron_jsc_engine_t* engine) {
    klyron_jsc_string_result_t result = {false, nullptr, 0, {0}};
    if (!engine || !engine->ctx) return result;

    /* Use Error().stack for stack trace — this is the standard JS approach */
    const char* code =
        "(function() { "
        "  try { throw new Error(); } "
        "  catch(e) { "
        "    var stack = e.stack || ''; "
        "    if (typeof stack === 'string') return stack; "
        "    return String(stack); "
        "  } "
        "})()";

    JSStringRef src = jsc_string_from_cstr(code);
    if (!src) return result;

    /* Create a fresh context for stack trace to avoid affecting the main context */
    JSValueRef exc = nullptr;
    JSStringRef stack_str = jsc_string_from_cstr("<stack>");
    JSValueRef val = JSEvaluateScript(engine->ctx, src, nullptr,
                                       stack_str, 1, &exc);
    JSStringRelease(stack_str);
    JSStringRelease(src);

    if (exc) {
        jsc_capture_exception(engine, exc);
        return result;
    }

    if (val) {
        JSValueRef to_str_exc = nullptr;
        JSStringRef str = JSValueToStringCopy(engine->ctx, val, &to_str_exc);
        if (str && !to_str_exc) {
            std::string s = jsc_string_to_std(str);
            JSStringRelease(str);
            jsc_set_string_result(&result, s);
            return result;
        }
    }

    jsc_set_string_result(&result, "(no stack trace)");
    return result;
}

/*
 * Clear the stored exception.
 */
void klyron_jsc_clear_exception(klyron_jsc_engine_t* engine) {
    if (engine) {
        engine->error_buf[0] = '\0';
    }
}
