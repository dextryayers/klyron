#include "klyron_jsc.h"
#include "cpp/impl/internal.h"

/*
 * JSC native ArrayBuffer and TypedArray support via C API.
 * Uses JSObjectMakeArrayBufferWithBytesNoCopy, JSObjectMakeTypedArray, etc.
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
    JSValueRef exc = nullptr;

    /* Check if it's an ArrayBuffer directly */
    if (JSValueGetTypedArrayType(engine->ctx, obj, &exc) == kJSTypedArrayTypeArrayBuffer ||
        JSValueGetTypedArrayType(engine->ctx, obj, &exc) == kJSTypedArrayTypeNone) {
        size_t length = JSObjectGetArrayBufferByteLength(engine->ctx, obj, &exc);
        if (exc || length == 0) return nullptr;
        void* ptr = JSObjectGetArrayBufferBytesPtr(engine->ctx, obj, &exc);
        if (exc || !ptr) return nullptr;
        *out_length = length;
        unsigned char* result = (unsigned char*)std::malloc(length);
        if (result) std::memcpy(result, ptr, length);
        return result;
    }

    /* It's a TypedArray — get its backing buffer */
    JSObjectRef buf = JSObjectGetTypedArrayBuffer(engine->ctx, obj, &exc);
    if (exc || !buf) return nullptr;

    size_t length = JSObjectGetArrayBufferByteLength(engine->ctx, buf, &exc);
    if (exc || length == 0) return nullptr;
    void* ptr = JSObjectGetArrayBufferBytesPtr(engine->ctx, buf, &exc);
    if (exc || !ptr) return nullptr;
    *out_length = length;
    unsigned char* result = (unsigned char*)std::malloc(length);
    if (result) std::memcpy(result, ptr, length);
    return result;
}

klyron_jsc_value_t* klyron_jsc_typed_array_new(klyron_jsc_engine_t* engine,
                                                 const char* type,
                                                 size_t length) {
    if (!engine || !engine->ctx || !type) return nullptr;

    /* Map type name to JSTypedArrayType */
    struct TypeMap { const char* name; JSTypedArrayType type; };
    static const TypeMap type_map[] = {
        {"Int8Array",         kJSTypedArrayTypeInt8Array},
        {"Int16Array",        kJSTypedArrayTypeInt16Array},
        {"Int32Array",        kJSTypedArrayTypeInt32Array},
        {"Uint8Array",        kJSTypedArrayTypeUint8Array},
        {"Uint8ClampedArray", kJSTypedArrayTypeUint8ClampedArray},
        {"Uint16Array",       kJSTypedArrayTypeUint16Array},
        {"Uint32Array",       kJSTypedArrayTypeUint32Array},
        {"Float32Array",      kJSTypedArrayTypeFloat32Array},
        {"Float64Array",      kJSTypedArrayTypeFloat64Array},
        {"BigInt64Array",     kJSTypedArrayTypeBigInt64Array},
        {"BigUint64Array",    kJSTypedArrayTypeBigUint64Array},
    };

    JSTypedArrayType arrayType = kJSTypedArrayTypeNone;
    for (const auto& m : type_map) {
        if (std::strcmp(type, m.name) == 0) {
            arrayType = m.type;
            break;
        }
    }

    if (arrayType == kJSTypedArrayTypeNone || arrayType == kJSTypedArrayTypeArrayBuffer) {
        jsc_set_error(engine, std::string("unknown TypedArray type: ") + type);
        return nullptr;
    }

    JSValueRef exc = nullptr;
    JSObjectRef val = JSObjectMakeTypedArray(engine->ctx, arrayType, length, &exc);
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
    JSValueRef exc = nullptr;
    size_t len = JSObjectGetTypedArrayLength(engine->ctx, obj, &exc);
    if (exc) jsc_capture_exception(engine, exc);
    return len;
}

klyron_jsc_value_t* klyron_jsc_typed_array_get_buffer(klyron_jsc_engine_t* engine,
                                                        klyron_jsc_value_t* value) {
    if (!engine || !value) return nullptr;
    if (!JSValueIsObject(engine->ctx, value->value)) return nullptr;

    JSObjectRef obj = (JSObjectRef)value->value;
    JSValueRef exc = nullptr;
    JSObjectRef buf = JSObjectGetTypedArrayBuffer(engine->ctx, obj, &exc);
    if (exc || !buf) {
        jsc_capture_exception(engine, exc);
        return nullptr;
    }

    auto v = new klyron_jsc_value_t(engine->ctx, buf);
    v->protect();
    return v;
}
