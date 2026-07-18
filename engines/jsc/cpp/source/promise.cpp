#include "klyron_jsc.h"
#include "cpp/impl/internal.h"

/*
 * Native JSC promise support using JSObjectMakeDeferredPromise (macOS 10.15+).
 * On JSC 2.52 (WebKitGTK), this API is available.
 * We store resolve/reject functions as hidden properties on the promise object.
 */

klyron_jsc_value_t* klyron_jsc_promise_new(klyron_jsc_engine_t* engine) {
    if (!engine || !engine->ctx) return nullptr;

    JSObjectRef resolve = nullptr;
    JSObjectRef reject = nullptr;
    JSValueRef exc = nullptr;

    JSObjectRef promise = JSObjectMakeDeferredPromise(engine->ctx, &resolve, &reject, &exc);
    if (exc || !promise) {
        jsc_capture_exception(engine, exc);
        /*
         * Fallback: if JSObjectMakeDeferredPromise is unavailable, use JS-level workaround.
         */
        const char* code =
            "(function() { "
            "  var resolve, reject; "
            "  var p = new Promise(function(rs, rj) { resolve = rs; reject = rj; }); "
            "  p._resolve = resolve; "
            "  p._reject = reject; "
            "  return p; "
            "})()";
        JSStringRef src = jsc_string_from_cstr(code);
        if (!src) return nullptr;
        exc = nullptr;
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

    /*
     * Store resolve and reject as hidden properties on the promise so
     * we can find them later in resolve/reject calls.
     * We use JSObjectSetProperty with DontEnum|DontDelete to make them hidden.
     */
    JSStringRef resolve_str = jsc_string_from_cstr("__klyron_resolve__");
    JSStringRef reject_str = jsc_string_from_cstr("__klyron_reject__");
    JSObjectSetProperty(engine->ctx, promise, resolve_str, resolve,
                        kJSPropertyAttributeDontEnum | kJSPropertyAttributeDontDelete | kJSPropertyAttributeReadOnly,
                        &exc);
    JSObjectSetProperty(engine->ctx, promise, reject_str, reject,
                        kJSPropertyAttributeDontEnum | kJSPropertyAttributeDontDelete | kJSPropertyAttributeReadOnly,
                        &exc);
    JSStringRelease(resolve_str);
    JSStringRelease(reject_str);

    auto v = new klyron_jsc_value_t(engine->ctx, promise);
    v->protect();
    return v;
}

klyron_jsc_result_t klyron_jsc_promise_resolve(klyron_jsc_engine_t* engine,
                                                klyron_jsc_value_t* promise,
                                                klyron_jsc_value_t* value) {
    klyron_jsc_result_t result = {false, {0}};
    if (!engine || !promise) return result;

    if (!JSValueIsObject(engine->ctx, promise->value)) return result;
    JSObjectRef obj = (JSObjectRef)promise->value;

    /* Try native resolve function stored as hidden property */
    JSStringRef resolve_str = jsc_string_from_cstr("__klyron_resolve__");
    JSValueRef exc = nullptr;
    JSValueRef resolve_fn = JSObjectGetProperty(engine->ctx, obj, resolve_str, &exc);
    JSStringRelease(resolve_str);

    if (!exc && resolve_fn && JSObjectIsFunction(engine->ctx, (JSObjectRef)resolve_fn)) {
        JSValueRef val = value ? value->value : JSValueMakeUndefined(engine->ctx);
        JSValueRef args[] = { val };
        exc = nullptr;
        JSObjectCallAsFunction(engine->ctx, (JSObjectRef)resolve_fn, obj, 1, args, &exc);
        if (!exc) {
            jsc_set_bool_result(&result, true);
            return result;
        }
    }

    /* Fallback: try using _resolve property (JS-level workaround) */
    JSStringRef fallback_str = jsc_string_from_cstr("_resolve");
    exc = nullptr;
    resolve_fn = JSObjectGetProperty(engine->ctx, obj, fallback_str, &exc);
    JSStringRelease(fallback_str);

    if (!exc && resolve_fn && JSObjectIsFunction(engine->ctx, (JSObjectRef)resolve_fn)) {
        JSValueRef val = value ? value->value : JSValueMakeUndefined(engine->ctx);
        JSValueRef args[] = { val };
        exc = nullptr;
        JSObjectCallAsFunction(engine->ctx, (JSObjectRef)resolve_fn, obj, 1, args, &exc);
        if (!exc) {
            jsc_set_bool_result(&result, true);
            return result;
        }
    }

    jsc_capture_exception(engine, exc);
    return result;
}

klyron_jsc_result_t klyron_jsc_promise_reject(klyron_jsc_engine_t* engine,
                                               klyron_jsc_value_t* promise,
                                               const char* reason) {
    klyron_jsc_result_t result = {false, {0}};
    if (!engine || !promise) return result;

    if (!JSValueIsObject(engine->ctx, promise->value)) return result;
    JSObjectRef obj = (JSObjectRef)promise->value;

    JSStringRef reject_str = jsc_string_from_cstr("__klyron_reject__");
    JSValueRef exc = nullptr;
    JSValueRef reject_fn = JSObjectGetProperty(engine->ctx, obj, reject_str, &exc);
    JSStringRelease(reject_str);

    JSStringRef reason_str = jsc_string_from_cstr(reason ? reason : "rejected");
    JSValueRef reason_val = JSValueMakeString(engine->ctx, reason_str);
    JSStringRelease(reason_str);

    if (!exc && reject_fn && JSObjectIsFunction(engine->ctx, (JSObjectRef)reject_fn)) {
        JSValueRef args[] = { reason_val };
        exc = nullptr;
        JSObjectCallAsFunction(engine->ctx, (JSObjectRef)reject_fn, obj, 1, args, &exc);
        if (!exc) {
            jsc_set_bool_result(&result, true);
            return result;
        }
    }

    /* Fallback: _reject */
    JSStringRef fallback_str = jsc_string_from_cstr("_reject");
    exc = nullptr;
    reject_fn = JSObjectGetProperty(engine->ctx, obj, fallback_str, &exc);
    JSStringRelease(fallback_str);

    if (!exc && reject_fn && JSObjectIsFunction(engine->ctx, (JSObjectRef)reject_fn)) {
        JSValueRef args[] = { reason_val };
        exc = nullptr;
        JSObjectCallAsFunction(engine->ctx, (JSObjectRef)reject_fn, obj, 1, args, &exc);
        if (!exc) {
            jsc_set_bool_result(&result, true);
            return result;
        }
    }

    jsc_capture_exception(engine, exc);
    return result;
}
