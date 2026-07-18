#include "klyron_jsc.h"
#include "cpp/impl/internal.h"
#include <vector>

/* ─── Global object access ────────────────────────────────────── */

klyron_jsc_result_t klyron_jsc_set_global(klyron_jsc_engine_t* engine,
                                           const char* name,
                                           klyron_jsc_value_t* value) {
    klyron_jsc_result_t result = {false, {0}};
    if (!engine || !engine->ctx || !name || !value) return result;

    JSObjectRef global = JSContextGetGlobalObject(engine->ctx);
    JSStringRef prop = jsc_string_from_cstr(name);
    if (!prop) return result;

    JSValueRef exc = nullptr;
    JSObjectSetProperty(engine->ctx, global, prop, value->value,
                        kJSPropertyAttributeNone, &exc);
    JSStringRelease(prop);

    if (exc) {
        jsc_capture_exception(engine, exc);
        return result;
    }

    jsc_set_bool_result(&result, true);
    return result;
}

klyron_jsc_value_t* klyron_jsc_get_global(klyron_jsc_engine_t* engine,
                                            const char* name) {
    if (!engine || !engine->ctx || !name) return nullptr;

    JSObjectRef global = JSContextGetGlobalObject(engine->ctx);
    JSStringRef prop = jsc_string_from_cstr(name);
    if (!prop) return nullptr;

    JSValueRef exc = nullptr;
    JSValueRef val = JSObjectGetProperty(engine->ctx, global, prop, &exc);
    JSStringRelease(prop);

    if (exc || !val) {
        jsc_capture_exception(engine, exc);
        return nullptr;
    }

    auto v = new klyron_jsc_value_t(engine->ctx, val);
    v->protect();
    return v;
}

/* ─── Object property access ──────────────────────────────────── */

klyron_jsc_result_t klyron_jsc_object_set_property(klyron_jsc_engine_t* engine,
                                                     klyron_jsc_value_t* object,
                                                     const char* name,
                                                     klyron_jsc_value_t* value) {
    klyron_jsc_result_t result = {false, {0}};
    if (!engine || !engine->ctx || !object || !name || !value) return result;
    if (!JSValueIsObject(engine->ctx, object->value)) return result;

    JSObjectRef obj = (JSObjectRef)object->value;
    JSStringRef prop = jsc_string_from_cstr(name);
    if (!prop) return result;

    JSValueRef exc = nullptr;
    JSObjectSetProperty(engine->ctx, obj, prop, value->value,
                        kJSPropertyAttributeNone, &exc);
    JSStringRelease(prop);

    if (exc) {
        jsc_capture_exception(engine, exc);
        return result;
    }

    jsc_set_bool_result(&result, true);
    return result;
}

klyron_jsc_value_t* klyron_jsc_object_get_property(klyron_jsc_engine_t* engine,
                                                     klyron_jsc_value_t* object,
                                                     const char* name) {
    if (!engine || !engine->ctx || !object || !name) return nullptr;
    if (!JSValueIsObject(engine->ctx, object->value)) return nullptr;

    JSObjectRef obj = (JSObjectRef)object->value;
    JSStringRef prop = jsc_string_from_cstr(name);
    if (!prop) return nullptr;

    JSValueRef exc = nullptr;
    JSValueRef val = JSObjectGetProperty(engine->ctx, obj, prop, &exc);
    JSStringRelease(prop);

    if (exc || !val) {
        jsc_capture_exception(engine, exc);
        return nullptr;
    }

    auto v = new klyron_jsc_value_t(engine->ctx, val);
    v->protect();
    return v;
}

/* ─── Call function ───────────────────────────────────────────── */

klyron_jsc_value_t* klyron_jsc_call_function(klyron_jsc_engine_t* engine,
                                               klyron_jsc_value_t* func,
                                               klyron_jsc_value_t* this_obj,
                                               int argc,
                                               klyron_jsc_value_t** argv) {
    if (!engine || !engine->ctx || !func) return nullptr;
    if (!JSValueIsObject(engine->ctx, func->value)) return nullptr;
    if (!JSObjectIsFunction(engine->ctx, (JSObjectRef)func->value)) {
        jsc_set_error(engine, "call_function: value is not a function");
        return nullptr;
    }

    JSObjectRef func_obj = (JSObjectRef)func->value;
    JSObjectRef this_val = this_obj && JSValueIsObject(engine->ctx, this_obj->value)
        ? (JSObjectRef)this_obj->value : JSContextGetGlobalObject(engine->ctx);

    std::vector<JSValueRef> args;
    for (int i = 0; i < argc; i++) {
        args.push_back(argv[i] ? argv[i]->value : JSValueMakeUndefined(engine->ctx));
    }

    JSValueRef exc = nullptr;
    JSValueRef result = JSObjectCallAsFunction(engine->ctx, func_obj,
                                                this_val, argc, args.data(), &exc);
    if (exc) {
        jsc_capture_exception(engine, exc);
        return nullptr;
    }

    auto v = new klyron_jsc_value_t(engine->ctx, result ? result : JSValueMakeUndefined(engine->ctx));
    v->protect();
    return v;
}
