#include "klyron_jsc.h"
#include "cpp/impl/internal.h"

/*
 * JSC C API does not expose Promise resolvers directly.
 * We create a deferred via JS-level Promise constructor.
 * The promise value is the JS Promise object itself.
 */

klyron_jsc_value_t* klyron_jsc_promise_new(klyron_jsc_engine_t* engine) {
    if (!engine || !engine->ctx) return nullptr;

    const char* code =
        "(function() { "
        "  var resolve, reject; "
        "  var p = new Promise(function(rs, rj) { resolve = rs; reject = rj; }); "
        "  p.resolve = resolve; "
        "  p.reject = reject; "
        "  return p; "
        "})()";

    JSStringRef src = jsc_string_from_cstr(code);
    if (!src) return nullptr;

    JSValueRef exc = nullptr;
    JSValueRef val = JSEvaluateScript(engine->ctx, src, nullptr,
                                       jsc_string_from_cstr("<promise>"), 1, &exc);
    JSStringRelease(src);

    if (exc || !val) {
        jsc_capture_exception(engine, exc);
        return nullptr;
    }

    auto v = new klyron_jsc_value_t(engine->ctx, val);
    v->protect();
    return v;
}

klyron_jsc_result_t klyron_jsc_promise_resolve(klyron_jsc_engine_t* engine,
                                                klyron_jsc_value_t* promise,
                                                klyron_jsc_value_t* value) {
    klyron_jsc_result_t result = {false, {0}};
    if (!engine || !promise) return result;

    JSObjectRef obj = (JSObjectRef)promise->value;
    JSStringRef resolve_prop = jsc_string_from_cstr("resolve");
    JSValueRef exc = nullptr;
    JSValueRef resolve_fn = JSObjectGetProperty(engine->ctx, obj, resolve_prop, &exc);
    JSStringRelease(resolve_prop);

    if (exc || !resolve_fn || !JSObjectIsFunction(engine->ctx, (JSObjectRef)resolve_fn)) {
        jsc_set_error(engine, "promise_resolve: no resolve function");
        return result;
    }

    JSValueRef val = value ? value->value : JSValueMakeUndefined(engine->ctx);
    JSValueRef call_args[] = { val };
    JSValueRef call_exc = nullptr;
    JSObjectCallAsFunction(engine->ctx, (JSObjectRef)resolve_fn, obj, 1, call_args, &call_exc);

    if (call_exc) {
        jsc_capture_exception(engine, call_exc);
        return result;
    }

    jsc_set_bool_result(&result, true);
    return result;
}

klyron_jsc_result_t klyron_jsc_promise_reject(klyron_jsc_engine_t* engine,
                                               klyron_jsc_value_t* promise,
                                               const char* reason) {
    klyron_jsc_result_t result = {false, {0}};
    if (!engine || !promise) return result;

    JSObjectRef obj = (JSObjectRef)promise->value;
    JSStringRef reject_prop = jsc_string_from_cstr("reject");
    JSValueRef exc = nullptr;
    JSValueRef reject_fn = JSObjectGetProperty(engine->ctx, obj, reject_prop, &exc);
    JSStringRelease(reject_prop);

    if (exc || !reject_fn || !JSObjectIsFunction(engine->ctx, (JSObjectRef)reject_fn)) {
        jsc_set_error(engine, "promise_reject: no reject function");
        return result;
    }

    JSStringRef reason_str = jsc_string_from_cstr(reason ? reason : "rejected");
    JSValueRef reason_val = JSValueMakeString(engine->ctx, reason_str);
    JSStringRelease(reason_str);

    JSValueRef call_args[] = { reason_val };
    JSValueRef call_exc = nullptr;
    JSObjectCallAsFunction(engine->ctx, (JSObjectRef)reject_fn, obj, 1, call_args, &call_exc);

    if (call_exc) {
        jsc_capture_exception(engine, call_exc);
        return result;
    }

    jsc_set_bool_result(&result, true);
    return result;
}
