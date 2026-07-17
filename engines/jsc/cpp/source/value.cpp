#include "klyron_jsc.h"
#include "cpp/impl/internal.h"

/* ─── Value creation ──────────────────────────────────────────── */

klyron_jsc_value_t* klyron_jsc_value_new_string(klyron_jsc_engine_t* engine,
                                                  const char* str) {
    if (!engine || !engine->ctx || !str) return nullptr;
    JSStringRef jsstr = jsc_string_from_cstr(str);
    if (!jsstr) return nullptr;
    JSValueRef val = JSValueMakeString(engine->ctx, jsstr);
    JSStringRelease(jsstr);
    auto v = new klyron_jsc_value_t(engine->ctx, val);
    v->protect();
    return v;
}

klyron_jsc_value_t* klyron_jsc_value_new_number(klyron_jsc_engine_t* engine,
                                                  double num) {
    if (!engine || !engine->ctx) return nullptr;
    JSValueRef val = JSValueMakeNumber(engine->ctx, num);
    auto v = new klyron_jsc_value_t(engine->ctx, val);
    v->protect();
    return v;
}

klyron_jsc_value_t* klyron_jsc_value_new_bool(klyron_jsc_engine_t* engine,
                                                bool val) {
    if (!engine || !engine->ctx) return nullptr;
    JSValueRef v = JSValueMakeBoolean(engine->ctx, val);
    auto w = new klyron_jsc_value_t(engine->ctx, v);
    w->protect();
    return w;
}

klyron_jsc_value_t* klyron_jsc_value_new_null(klyron_jsc_engine_t* engine) {
    if (!engine || !engine->ctx) return nullptr;
    JSValueRef val = JSValueMakeNull(engine->ctx);
    auto v = new klyron_jsc_value_t(engine->ctx, val);
    v->protect();
    return v;
}

klyron_jsc_value_t* klyron_jsc_value_new_undefined(klyron_jsc_engine_t* engine) {
    if (!engine || !engine->ctx) return nullptr;
    JSValueRef val = JSValueMakeUndefined(engine->ctx);
    auto v = new klyron_jsc_value_t(engine->ctx, val);
    v->protect();
    return v;
}

klyron_jsc_value_t* klyron_jsc_value_new_object(klyron_jsc_engine_t* engine) {
    if (!engine || !engine->ctx) return nullptr;
    JSValueRef val = JSObjectMake(engine->ctx, nullptr, nullptr);
    auto v = new klyron_jsc_value_t(engine->ctx, val);
    v->protect();
    return v;
}

klyron_jsc_value_t* klyron_jsc_value_new_array(klyron_jsc_engine_t* engine) {
    if (!engine || !engine->ctx) return nullptr;
    JSValueRef val = JSObjectMakeArray(engine->ctx, 0, nullptr, nullptr);
    auto v = new klyron_jsc_value_t(engine->ctx, val);
    v->protect();
    return v;
}

klyron_jsc_value_t* klyron_jsc_value_new_symbol(klyron_jsc_engine_t* engine,
                                                  const char* description) {
    if (!engine || !engine->ctx) return nullptr;
    JSStringRef desc = description ? jsc_string_from_cstr(description) : nullptr;
    JSValueRef val = JSValueMakeSymbol(engine->ctx, desc);
    if (desc) JSStringRelease(desc);
    auto v = new klyron_jsc_value_t(engine->ctx, val);
    v->protect();
    return v;
}

klyron_jsc_value_t* klyron_jsc_value_new_error(klyron_jsc_engine_t* engine,
                                                 const char* message) {
    if (!engine || !engine->ctx) return nullptr;
    std::string code = "new Error(";
    code += message ? "'" + std::string(message) + "'" : "'error'";
    code += ")";
    JSStringRef src = jsc_string_from_cstr(code.c_str());
    if (!src) return nullptr;
    JSValueRef exc = nullptr;
    JSValueRef val = JSEvaluateScript(engine->ctx, src, nullptr,
                                       jsc_string_from_cstr("<error>"), 1, &exc);
    JSStringRelease(src);
    if (exc || !val) {
        jsc_capture_exception(engine, exc);
        return nullptr;
    }
    auto v = new klyron_jsc_value_t(engine->ctx, val);
    v->protect();
    return v;
}

/* ─── Value inspection ────────────────────────────────────────── */

klyron_jsc_type_result_t klyron_jsc_value_typeof(klyron_jsc_engine_t* engine,
                                                   klyron_jsc_value_t* value) {
    klyron_jsc_type_result_t result = {KLYRON_JSC_UNDEFINED, false, {0}};
    if (!engine || !value) return result;

    JSValueRef val = value->value;
    result.success = true;

    if (JSValueIsUndefined(engine->ctx, val))       result.type = KLYRON_JSC_UNDEFINED;
    else if (JSValueIsNull(engine->ctx, val))       result.type = KLYRON_JSC_NULL;
    else if (JSValueIsBoolean(engine->ctx, val))    result.type = KLYRON_JSC_BOOLEAN;
    else if (JSValueIsNumber(engine->ctx, val))     result.type = KLYRON_JSC_NUMBER;
    else if (JSValueIsString(engine->ctx, val))     result.type = KLYRON_JSC_STRING;
    else if (JSValueIsSymbol(engine->ctx, val))     result.type = KLYRON_JSC_SYMBOL;
    else if (JSObjectIsFunction(engine->ctx, (JSObjectRef)val)) result.type = KLYRON_JSC_FUNCTION;
    else if (JSValueIsArray(engine->ctx, val))      result.type = KLYRON_JSC_ARRAY;
    else if (JSValueIsObject(engine->ctx, val)) {
        JSStringRef err_str = jsc_string_from_cstr("Error");
        JSValueRef error_ctor = JSObjectGetProperty(engine->ctx,
            JSContextGetGlobalObject(engine->ctx), err_str, nullptr);
        JSStringRelease(err_str);
        bool is_err = false;
        if (error_ctor && JSValueIsObject(engine->ctx, error_ctor)) {
            JSValueRef instance_of = JSValueIsInstanceOf(engine->ctx, val,
                (JSObjectRef)error_ctor, nullptr);
            is_err = instance_of;
        }
        result.type = is_err ? KLYRON_JSC_ERROR : KLYRON_JSC_OBJECT;
    }
    else                                             result.type = KLYRON_JSC_OBJECT;

    return result;
}

klyron_jsc_string_result_t klyron_jsc_value_to_string(klyron_jsc_engine_t* engine,
                                                        klyron_jsc_value_t* value) {
    klyron_jsc_string_result_t result = {false, nullptr, 0, {0}};
    if (!engine || !value) return result;

    if (JSValueIsUndefined(engine->ctx, value->value)) {
        jsc_set_string_result(&result, "undefined");
        return result;
    }
    if (JSValueIsNull(engine->ctx, value->value)) {
        jsc_set_string_result(&result, "null");
        return result;
    }

    JSValueRef exc = nullptr;
    JSStringRef str = JSValueToStringCopy(engine->ctx, value->value, &exc);
    if (exc || !str) {
        jsc_capture_exception(engine, exc);
        return result;
    }

    std::string s = jsc_string_to_std(str);
    JSStringRelease(str);
    jsc_set_string_result(&result, s);
    return result;
}

double klyron_jsc_value_to_number(klyron_jsc_engine_t* engine,
                                   klyron_jsc_value_t* value) {
    if (!engine || !value) return 0.0;
    JSValueRef exc = nullptr;
    double num = JSValueToNumber(engine->ctx, value->value, &exc);
    if (exc) jsc_capture_exception(engine, exc);
    return num;
}

bool klyron_jsc_value_to_bool(klyron_jsc_engine_t* engine,
                               klyron_jsc_value_t* value) {
    if (!engine || !value) return false;
    return JSValueToBoolean(engine->ctx, value->value);
}

bool klyron_jsc_value_is_array(klyron_jsc_engine_t* engine,
                                klyron_jsc_value_t* value) {
    if (!engine || !value) return false;
    return JSValueIsArray(engine->ctx, value->value);
}

bool klyron_jsc_value_is_function(klyron_jsc_engine_t* engine,
                                   klyron_jsc_value_t* value) {
    if (!engine || !value || !JSValueIsObject(engine->ctx, value->value)) return false;
    return JSObjectIsFunction(engine->ctx, (JSObjectRef)value->value);
}

bool klyron_jsc_value_is_object(klyron_jsc_engine_t* engine,
                                 klyron_jsc_value_t* value) {
    if (!engine || !value) return false;
    return JSValueIsObject(engine->ctx, value->value);
}

bool klyron_jsc_value_is_error(klyron_jsc_engine_t* engine,
                                klyron_jsc_value_t* value) {
    if (!engine || !value || !JSValueIsObject(engine->ctx, value->value)) return false;
    JSStringRef err_str = jsc_string_from_cstr("Error");
    JSValueRef error_ctor = JSObjectGetProperty(engine->ctx,
        JSContextGetGlobalObject(engine->ctx), err_str, nullptr);
    JSStringRelease(err_str);
    if (!error_ctor || !JSValueIsObject(engine->ctx, error_ctor)) return false;
    return JSValueIsInstanceOf(engine->ctx, value->value,
                               (JSObjectRef)error_ctor, nullptr);
}

bool klyron_jsc_value_is_symbol(klyron_jsc_engine_t* engine,
                                 klyron_jsc_value_t* value) {
    if (!engine || !value) return false;
    return JSValueIsSymbol(engine->ctx, value->value);
}

bool klyron_jsc_value_is_promise(klyron_jsc_engine_t* engine,
                                  klyron_jsc_value_t* value) {
    if (!engine || !value || !JSValueIsObject(engine->ctx, value->value)) return false;
    JSStringRef promise_str = jsc_string_from_cstr("Promise");
    JSValueRef promise_ctor = JSObjectGetProperty(engine->ctx,
        JSContextGetGlobalObject(engine->ctx), promise_str, nullptr);
    JSStringRelease(promise_str);
    if (!promise_ctor || !JSValueIsObject(engine->ctx, promise_ctor)) return false;
    return JSValueIsInstanceOf(engine->ctx, value->value,
                               (JSObjectRef)promise_ctor, nullptr);
}

bool klyron_jsc_value_is_typed_array(klyron_jsc_engine_t* engine,
                                      klyron_jsc_value_t* value) {
    if (!engine || !value || !JSValueIsObject(engine->ctx, value->value)) return false;
    JSStringRef ta_str = jsc_string_from_cstr("ArrayBuffer");
    JSValueRef ta_ctor = JSObjectGetProperty(engine->ctx,
        JSContextGetGlobalObject(engine->ctx), ta_str, nullptr);
    JSStringRelease(ta_str);
    if (!ta_ctor || !JSValueIsObject(engine->ctx, ta_ctor)) return false;
    return JSValueIsInstanceOf(engine->ctx, value->value,
                               (JSObjectRef)ta_ctor, nullptr);
}

bool klyron_jsc_value_is_null(klyron_jsc_engine_t* engine,
                               klyron_jsc_value_t* value) {
    if (!engine || !value) return false;
    return JSValueIsNull(engine->ctx, value->value);
}

bool klyron_jsc_value_is_undefined(klyron_jsc_engine_t* engine,
                                    klyron_jsc_value_t* value) {
    if (!engine || !value) return false;
    return JSValueIsUndefined(engine->ctx, value->value);
}

bool klyron_jsc_value_is_boolean(klyron_jsc_engine_t* engine,
                                  klyron_jsc_value_t* value) {
    if (!engine || !value) return false;
    return JSValueIsBoolean(engine->ctx, value->value);
}

bool klyron_jsc_value_is_number(klyron_jsc_engine_t* engine,
                                 klyron_jsc_value_t* value) {
    if (!engine || !value) return false;
    return JSValueIsNumber(engine->ctx, value->value);
}

bool klyron_jsc_value_is_string(klyron_jsc_engine_t* engine,
                                 klyron_jsc_value_t* value) {
    if (!engine || !value) return false;
    return JSValueIsString(engine->ctx, value->value);
}

void klyron_jsc_value_dispose(klyron_jsc_value_t* value) {
    delete value;
}
