#include "klyron_jsc.h"
#include "cpp/impl/internal.h"

/*
 * JSC C API supports ArrayBuffer via JSObjectMakeArrayBufferWithBytesNoCopy
 * and typed arrays. This implementation wraps those APIs.
 */

klyron_jsc_value_t* klyron_jsc_array_buffer_new(klyron_jsc_engine_t* engine,
                                                  const unsigned char* data,
                                                  size_t length) {
    if (!engine || !engine->ctx || !data || length == 0) return nullptr;

    void* bytes = std::malloc(length);
    if (!bytes) return nullptr;
    std::memcpy(bytes, data, length);

    JSValueRef exc = nullptr;
    JSValueRef buf = JSObjectMakeArrayBufferWithBytesNoCopy(
        engine->ctx, bytes, length,
        [](void* ptr, void* ctx) { std::free(ptr); },
        nullptr, &exc);

    if (exc || !buf) {
        std::free(bytes);
        jsc_capture_exception(engine, exc);
        return nullptr;
    }

    auto v = new klyron_jsc_value_t(engine->ctx, buf);
    v->protect();
    return v;
}

unsigned char* klyron_jsc_array_buffer_get_data(klyron_jsc_engine_t* engine,
                                                  klyron_jsc_value_t* value,
                                                  size_t* out_length) {
    if (!engine || !value || !out_length) return nullptr;
    if (!JSValueIsObject(engine->ctx, value->value)) return nullptr;

    JSObjectRef obj = (JSObjectRef)value->value;
    JSTypedArrayType type = JSValueGetTypedArrayType(engine->ctx, obj, nullptr);
    if (type == kJSTypedArrayTypeNone) {
        /* Try as ArrayBuffer directly */
        size_t length = JSObjectGetArrayBufferByteLength(engine->ctx, obj, nullptr);
        if (length == 0) return nullptr;
        void* ptr = JSObjectGetArrayBufferBytesPtr(engine->ctx, obj, nullptr);
        if (!ptr) return nullptr;
        *out_length = length;
        unsigned char* result = (unsigned char*)std::malloc(length);
        if (result) std::memcpy(result, ptr, length);
        return result;
    }

    /* Get the ArrayBuffer from the typed array */
    JSValueRef buf = JSObjectGetTypedArrayBuffer(engine->ctx, obj, nullptr);
    if (!buf) return nullptr;
    JSObjectRef buf_obj = (JSObjectRef)buf;
    size_t length = JSObjectGetArrayBufferByteLength(engine->ctx, buf_obj, nullptr);
    if (length == 0) return nullptr;
    void* ptr = JSObjectGetArrayBufferBytesPtr(engine->ctx, buf_obj, nullptr);
    if (!ptr) return nullptr;
    *out_length = length;
    unsigned char* result = (unsigned char*)std::malloc(length);
    if (result) std::memcpy(result, ptr, length);
    return result;
}

klyron_jsc_value_t* klyron_jsc_typed_array_new(klyron_jsc_engine_t* engine,
                                                 const char* type,
                                                 size_t length) {
    if (!engine || !engine->ctx || !type) return nullptr;

    std::string code = "new ";
    code += type;
    code += "(" + std::to_string(length) + ")";

    JSStringRef src = jsc_string_from_cstr(code.c_str());
    if (!src) return nullptr;

    JSValueRef exc = nullptr;
    JSValueRef val = JSEvaluateScript(engine->ctx, src, nullptr,
                                       jsc_string_from_cstr("<typedarray>"), 1, &exc);
    JSStringRelease(src);

    if (exc || !val) {
        jsc_capture_exception(engine, exc);
        return nullptr;
    }

    auto v = new klyron_jsc_value_t(engine->ctx, val);
    v->protect();
    return v;
}

size_t klyron_jsc_typed_array_get_length(klyron_jsc_engine_t* engine,
                                          klyron_jsc_value_t* value) {
    if (!engine || !value) return 0;
    if (!JSValueIsObject(engine->ctx, value->value)) return 0;

    JSObjectRef obj = (JSObjectRef)value->value;
    JSTypedArrayType type = JSValueGetTypedArrayType(engine->ctx, obj, nullptr);
    if (type == kJSTypedArrayTypeNone) return 0;

    return JSObjectGetTypedArrayByteLength(engine->ctx, obj, nullptr);
}

klyron_jsc_value_t* klyron_jsc_typed_array_get_buffer(klyron_jsc_engine_t* engine,
                                                        klyron_jsc_value_t* value) {
    if (!engine || !value) return nullptr;
    if (!JSValueIsObject(engine->ctx, value->value)) return nullptr;

    JSObjectRef obj = (JSObjectRef)value->value;
    JSTypedArrayType type = JSValueGetTypedArrayType(engine->ctx, obj, nullptr);
    if (type == kJSTypedArrayTypeNone) return nullptr;

    JSValueRef buf = JSObjectGetTypedArrayBuffer(engine->ctx, obj, nullptr);
    if (!buf) return nullptr;

    auto v = new klyron_jsc_value_t(engine->ctx, buf);
    v->protect();
    return v;
}
