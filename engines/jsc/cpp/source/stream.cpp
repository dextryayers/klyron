#include "klyron_jsc.h"
#include "cpp/impl/internal.h"
#include <cstring>
#include <cstdlib>
#include <vector>
#include <algorithm>
#include <climits>

struct StreamState {
    bool ended;
    bool errored;
    bool destroyed;
    bool writable;
    bool readable;
    size_t high_water_mark;
    std::string error_message;
};

static klyron_jsc_value_t* stream_create_state(klyron_jsc_engine_t* engine) {
    if (!engine) return nullptr;
    JSObjectRef obj = JSObjectMake(engine->ctx, nullptr, nullptr);
    auto set = [&](const char* name, JSValueRef val) {
        JSStringRef key = jsc_string_from_cstr(name);
        JSObjectSetProperty(engine->ctx, obj, key, val, kJSPropertyAttributeNone, nullptr);
        JSStringRelease(key);
    };
    set("ended", JSValueMakeBoolean(engine->ctx, false));
    set("errored", JSValueMakeBoolean(engine->ctx, false));
    set("destroyed", JSValueMakeBoolean(engine->ctx, false));
    set("writable", JSValueMakeBoolean(engine->ctx, true));
    set("readable", JSValueMakeBoolean(engine->ctx, true));
    set("highWaterMark", JSValueMakeNumber(engine->ctx, 16384));
    set("length", JSValueMakeNumber(engine->ctx, 0));
    auto v = new klyron_jsc_value_t(engine->ctx, obj);
    v->protect();
    return v;
}

klyron_jsc_value_t* klyron_jsc_stream_readable_new(klyron_jsc_engine_t* engine, klyron_jsc_value_t* options) {
    if (!engine) return nullptr;
    JSObjectRef obj = JSObjectMake(engine->ctx, nullptr, nullptr);
    auto set = [&](const char* name, JSValueRef val) {
        JSStringRef key = jsc_string_from_cstr(name);
        JSObjectSetProperty(engine->ctx, obj, key, val, kJSPropertyAttributeNone, nullptr);
        JSStringRelease(key);
    };
    auto get_bool = [&](const char* name, bool def) -> bool {
        if (!options || !JSValueIsObject(engine->ctx, options->value)) return def;
        JSStringRef key = jsc_string_from_cstr(name);
        JSValueRef exc = nullptr;
        JSValueRef val = JSObjectGetProperty(engine->ctx, (JSObjectRef)options->value, key, &exc);
        JSStringRelease(key);
        if (exc || !val) return def;
        return JSValueToBoolean(engine->ctx, val);
    };
    auto get_num = [&](const char* name, double def) -> double {
        if (!options || !JSValueIsObject(engine->ctx, options->value)) return def;
        JSStringRef key = jsc_string_from_cstr(name);
        JSValueRef exc = nullptr;
        JSValueRef val = JSObjectGetProperty(engine->ctx, (JSObjectRef)options->value, key, &exc);
        JSStringRelease(key);
        if (exc || !val) return def;
        return JSValueToNumber(engine->ctx, val, &exc);
    };
    set("readable", JSValueMakeBoolean(engine->ctx, true));
    set("writable", JSValueMakeBoolean(engine->ctx, false));
    set("objectMode", JSValueMakeBoolean(engine->ctx, get_bool("objectMode", false)));
    set("highWaterMark", JSValueMakeNumber(engine->ctx, get_num("highWaterMark", 16384)));
    set("encoding", JSValueMakeNull(engine->ctx));
    set("buffer", JSObjectMakeArray(engine->ctx, 0, nullptr, nullptr));
    set("ended", JSValueMakeBoolean(engine->ctx, false));
    set("destroyed", JSValueMakeBoolean(engine->ctx, false));
    set("errored", JSValueMakeBoolean(engine->ctx, false));
    auto v = new klyron_jsc_value_t(engine->ctx, obj);
    v->protect();
    return v;
}

klyron_jsc_value_t* klyron_jsc_stream_writable_new(klyron_jsc_engine_t* engine, klyron_jsc_value_t* options) {
    if (!engine) return nullptr;
    JSObjectRef obj = JSObjectMake(engine->ctx, nullptr, nullptr);
    auto set = [&](const char* name, JSValueRef val) {
        JSStringRef key = jsc_string_from_cstr(name);
        JSObjectSetProperty(engine->ctx, obj, key, val, kJSPropertyAttributeNone, nullptr);
        JSStringRelease(key);
    };
    set("readable", JSValueMakeBoolean(engine->ctx, false));
    set("writable", JSValueMakeBoolean(engine->ctx, true));
    set("ended", JSValueMakeBoolean(engine->ctx, false));
    set("destroyed", JSValueMakeBoolean(engine->ctx, false));
    set("errored", JSValueMakeBoolean(engine->ctx, false));
    set("highWaterMark", JSValueMakeNumber(engine->ctx, 16384));
    set("length", JSValueMakeNumber(engine->ctx, 0));
    auto v = new klyron_jsc_value_t(engine->ctx, obj);
    v->protect();
    return v;
}

klyron_jsc_value_t* klyron_jsc_stream_transform_new(klyron_jsc_engine_t* engine, klyron_jsc_value_t* options) {
    if (!engine) return nullptr;
    JSObjectRef obj = JSObjectMake(engine->ctx, nullptr, nullptr);
    auto set = [&](const char* name, JSValueRef val) {
        JSStringRef key = jsc_string_from_cstr(name);
        JSObjectSetProperty(engine->ctx, obj, key, val, kJSPropertyAttributeNone, nullptr);
        JSStringRelease(key);
    };
    set("readable", JSValueMakeBoolean(engine->ctx, true));
    set("writable", JSValueMakeBoolean(engine->ctx, true));
    set("ended", JSValueMakeBoolean(engine->ctx, false));
    set("destroyed", JSValueMakeBoolean(engine->ctx, false));
    set("errored", JSValueMakeBoolean(engine->ctx, false));
    auto v = new klyron_jsc_value_t(engine->ctx, obj);
    v->protect();
    return v;
}

klyron_jsc_result_t klyron_jsc_stream_push(klyron_jsc_engine_t* engine, klyron_jsc_value_t* stream, klyron_jsc_value_t* chunk) {
    klyron_jsc_result_t result = {false, {0}};
    if (!engine || !stream) return result;
    if (!JSValueIsObject(engine->ctx, stream->value)) return result;
    JSObjectRef obj = (JSObjectRef)stream->value;
    if (chunk) {
        JSStringRef buf_key = jsc_string_from_cstr("buffer");
        JSValueRef exc = nullptr;
        JSValueRef buf_val = JSObjectGetProperty(engine->ctx, obj, buf_key, &exc);
        JSStringRelease(buf_key);
        if (!exc && buf_val && JSValueIsObject(engine->ctx, buf_val)) {
            JSObjectRef buf_arr = (JSObjectRef)buf_val;
            JSStringRef len_key = jsc_string_from_cstr("length");
            JSValueRef len_val = JSObjectGetProperty(engine->ctx, buf_arr, len_key, &exc);
            JSStringRelease(len_key);
            if (!exc && len_val) {
                double len = JSValueToNumber(engine->ctx, len_val, &exc);
                JSStringRef idx_key = JSStringCreateWithUTF8CString(std::to_string((int)len).c_str());
                JSObjectSetProperty(engine->ctx, buf_arr, idx_key, chunk->value, kJSPropertyAttributeNone, &exc);
                JSStringRelease(idx_key);
            }
        }
    }
    result.success = true;
    return result;
}

klyron_jsc_value_t* klyron_jsc_stream_read(klyron_jsc_engine_t* engine, klyron_jsc_value_t* stream, size_t size) {
    if (!engine || !stream) return nullptr;
    if (!JSValueIsObject(engine->ctx, stream->value)) return nullptr;
    JSObjectRef obj = (JSObjectRef)stream->value;
    JSStringRef buf_key = jsc_string_from_cstr("buffer");
    JSValueRef exc = nullptr;
    JSValueRef buf_val = JSObjectGetProperty(engine->ctx, obj, buf_key, &exc);
    JSStringRelease(buf_key);
    if (exc || !buf_val || !JSValueIsObject(engine->ctx, buf_val)) return nullptr;
    JSObjectRef buf_arr = (JSObjectRef)buf_val;
    JSStringRef len_key = jsc_string_from_cstr("length");
    JSValueRef len_val = JSObjectGetProperty(engine->ctx, buf_arr, len_key, &exc);
    JSStringRelease(len_key);
    if (exc || !len_val) return nullptr;
    double len = JSValueToNumber(engine->ctx, len_val, &exc);
    if (exc || len == 0) return nullptr;
    size_t read_size = size > 0 ? std::min((size_t)len, size) : (size_t)len;
    JSStringRef idx_key = JSStringCreateWithUTF8CString("0");
    JSValueRef first = JSObjectGetProperty(engine->ctx, buf_arr, idx_key, &exc);
    JSStringRelease(idx_key);
    for (size_t i = 1; i < read_size && i < (size_t)len; i++) {
        std::string old_key = std::to_string(i);
        std::string new_key = std::to_string(i - 1);
        JSStringRef old_s = jsc_string_from_cstr(old_key.c_str());
        JSStringRef new_s = jsc_string_from_cstr(new_key.c_str());
        JSValueRef val = JSObjectGetProperty(engine->ctx, buf_arr, old_s, &exc);
        JSObjectSetProperty(engine->ctx, buf_arr, new_s, val, kJSPropertyAttributeNone, &exc);
        JSStringRelease(new_s);
        JSStringRelease(old_s);
    }
    JSObjectRef delete_key = JSObjectMake(engine->ctx, nullptr, nullptr);
    JSStringRef del_s = jsc_string_from_cstr(std::to_string((int)len - 1).c_str());
    JSObjectDeleteProperty(engine->ctx, buf_arr, del_s, &exc);
    JSStringRelease(del_s);
    JSStringRef new_len = jsc_string_from_cstr(std::to_string((int)len - (int)read_size).c_str());
    JSObjectSetProperty(engine->ctx, buf_arr, len_key, JSValueMakeNumber(engine->ctx, len - read_size), kJSPropertyAttributeNone, &exc);
    JSStringRelease(new_len);
    return first ? (new klyron_jsc_value_t(engine->ctx, first))->protect(), nullptr : nullptr;
}

klyron_jsc_result_t klyron_jsc_stream_end(klyron_jsc_engine_t* engine, klyron_jsc_value_t* stream) {
    klyron_jsc_result_t result = {false, {0}};
    if (!engine || !stream) return result;
    if (!JSValueIsObject(engine->ctx, stream->value)) return result;
    JSObjectRef obj = (JSObjectRef)stream->value;
    JSStringRef key = jsc_string_from_cstr("ended");
    JSObjectSetProperty(engine->ctx, obj, key, JSValueMakeBoolean(engine->ctx, true), kJSPropertyAttributeNone, nullptr);
    JSStringRelease(key);
    result.success = true;
    return result;
}

klyron_jsc_result_t klyron_jsc_stream_destroy(klyron_jsc_engine_t* engine, klyron_jsc_value_t* stream) {
    klyron_jsc_result_t result = {false, {0}};
    if (!engine || !stream) return result;
    if (!JSValueIsObject(engine->ctx, stream->value)) return result;
    JSObjectRef obj = (JSObjectRef)stream->value;
    JSStringRef key = jsc_string_from_cstr("destroyed");
    JSObjectSetProperty(engine->ctx, obj, key, JSValueMakeBoolean(engine->ctx, true), kJSPropertyAttributeNone, nullptr);
    JSStringRelease(key);
    result.success = true;
    return result;
}

klyron_jsc_value_t* klyron_jsc_stream_pipe(klyron_jsc_engine_t* engine, klyron_jsc_value_t* readable, klyron_jsc_value_t* writable) {
    if (!engine || !readable || !writable) return nullptr;
    if (!JSValueIsObject(engine->ctx, readable->value) || !JSValueIsObject(engine->ctx, writable->value)) return nullptr;
    auto v = new klyron_jsc_value_t(engine->ctx, writable->value);
    v->protect();
    return v;
}
