#include "klyron_jsc.h"
#include "cpp/impl/internal.h"

klyron_jsc_string_result_t klyron_jsc_json_stringify(klyron_jsc_engine_t* engine,
                                                       klyron_jsc_value_t* value) {
    klyron_jsc_string_result_t result = {false, nullptr, 0, {0}};
    if (!engine || !value) return result;

    JSValueRef exc = nullptr;
    JSStringRef str = JSValueCreateJSONString(engine->ctx, value->value, 0, &exc);
    if (exc || !str) {
        jsc_capture_exception(engine, exc);
        return result;
    }

    std::string s = jsc_string_to_std(str);
    JSStringRelease(str);
    jsc_set_string_result(&result, s);
    return result;
}

klyron_jsc_value_t* klyron_jsc_json_parse(klyron_jsc_engine_t* engine,
                                           const char* json) {
    if (!engine || !engine->ctx || !json) return nullptr;

    JSStringRef jsstr = jsc_string_from_cstr(json);
    if (!jsstr) return nullptr;

    JSValueRef exc = nullptr;
    JSValueRef val = JSValueMakeFromJSONString(engine->ctx, jsstr);
    JSStringRelease(jsstr);

    if (exc || !val) {
        jsc_capture_exception(engine, exc);
        return nullptr;
    }

    auto v = new klyron_jsc_value_t(engine->ctx, val);
    v->protect();
    return v;
}
