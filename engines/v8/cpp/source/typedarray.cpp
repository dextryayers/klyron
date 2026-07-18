#include "klyron_v8.h"
#include "cpp/impl/internal.h"

#include <cstring>

klyron_v8_typed_array_type_t klyron_v8_get_typed_array_type(
    klyron_v8_context_t* ctx, klyron_v8_value_t* value) {
    if (!ctx || !value) return KLYRON_V8_TYPED_ARRAY_NONE;
    auto iso = get_iso(ctx);
    v8::HandleScope scope(iso);
    auto val = value->value->Get(iso);
    if (!val->IsTypedArray()) return KLYRON_V8_TYPED_ARRAY_NONE;

    if (val->IsInt8Array())         return KLYRON_V8_TYPED_ARRAY_INT8;
    if (val->IsUint8Array())        return KLYRON_V8_TYPED_ARRAY_UINT8;
    if (val->IsUint8ClampedArray()) return KLYRON_V8_TYPED_ARRAY_UINT8_CLAMPED;
    if (val->IsInt16Array())        return KLYRON_V8_TYPED_ARRAY_INT16;
    if (val->IsUint16Array())       return KLYRON_V8_TYPED_ARRAY_UINT16;
    if (val->IsInt32Array())        return KLYRON_V8_TYPED_ARRAY_INT32;
    if (val->IsUint32Array())       return KLYRON_V8_TYPED_ARRAY_UINT32;
    if (val->IsFloat16Array())      return KLYRON_V8_TYPED_ARRAY_FLOAT16;
    if (val->IsFloat32Array())      return KLYRON_V8_TYPED_ARRAY_FLOAT32;
    if (val->IsFloat64Array())      return KLYRON_V8_TYPED_ARRAY_FLOAT64;
    if (val->IsBigInt64Array())     return KLYRON_V8_TYPED_ARRAY_BIGINT64;
    if (val->IsBigUint64Array())    return KLYRON_V8_TYPED_ARRAY_BIGUINT64;
    return KLYRON_V8_TYPED_ARRAY_NONE;
}

static v8::Local<v8::Value> create_typed_array_by_type(
    v8::Isolate* iso, const char* type, v8::Local<v8::ArrayBuffer> ab,
    size_t byte_offset, size_t length) {

    if (std::strcmp(type, "Int8Array") == 0)
        return v8::Int8Array::New(ab, byte_offset, length);
    if (std::strcmp(type, "Uint8Array") == 0)
        return v8::Uint8Array::New(ab, byte_offset, length);
    if (std::strcmp(type, "Uint8ClampedArray") == 0)
        return v8::Uint8ClampedArray::New(ab, byte_offset, length);
    if (std::strcmp(type, "Int16Array") == 0)
        return v8::Int16Array::New(ab, byte_offset, length);
    if (std::strcmp(type, "Uint16Array") == 0)
        return v8::Uint16Array::New(ab, byte_offset, length);
    if (std::strcmp(type, "Int32Array") == 0)
        return v8::Int32Array::New(ab, byte_offset, length);
    if (std::strcmp(type, "Uint32Array") == 0)
        return v8::Uint32Array::New(ab, byte_offset, length);
    if (std::strcmp(type, "Float16Array") == 0)
        return v8::Float16Array::New(ab, byte_offset, length);
    if (std::strcmp(type, "Float32Array") == 0)
        return v8::Float32Array::New(ab, byte_offset, length);
    if (std::strcmp(type, "Float64Array") == 0)
        return v8::Float64Array::New(ab, byte_offset, length);
    if (std::strcmp(type, "BigInt64Array") == 0)
        return v8::BigInt64Array::New(ab, byte_offset, length);
    if (std::strcmp(type, "BigUint64Array") == 0)
        return v8::BigUint64Array::New(ab, byte_offset, length);
    return v8::Local<v8::Value>();
}

klyron_v8_value_t* klyron_v8_typed_array_new(klyron_v8_context_t* ctx,
                                               const char* type,
                                               size_t length) {
    if (!ctx || !type) return nullptr;
    auto iso = get_iso(ctx);
    v8::Locker locker(iso);
    v8::Isolate::Scope iso_scope(iso);
    v8::HandleScope scope(iso);
    auto context = get_ctx(ctx);
    v8::Context::Scope ctx_scope(context);

    size_t byte_length = length * 8;
    auto ab = v8::ArrayBuffer::New(iso, byte_length);
    auto ta = create_typed_array_by_type(iso, type, ab, 0, length);
    if (ta.IsEmpty()) return nullptr;

    return new klyron_v8_value(iso, ta, ctx->parent);
}

size_t klyron_v8_typed_array_get_length(klyron_v8_context_t* ctx,
                                         klyron_v8_value_t* value) {
    if (!ctx || !value) return 0;
    auto iso = get_iso(ctx);
    v8::HandleScope scope(iso);
    auto val = value->value->Get(iso);
    if (!val->IsTypedArray()) return 0;
    return val.As<v8::TypedArray>()->Length();
}

klyron_v8_value_t* klyron_v8_typed_array_get_buffer(klyron_v8_context_t* ctx,
                                                     klyron_v8_value_t* value) {
    if (!ctx || !value) return nullptr;
    auto iso = get_iso(ctx);
    v8::HandleScope scope(iso);
    auto val = value->value->Get(iso);
    if (!val->IsTypedArray()) return nullptr;
    auto buf = val.As<v8::TypedArray>()->Buffer();
    return new klyron_v8_value(iso, buf, ctx->parent);
}

klyron_v8_value_t* klyron_v8_array_buffer_new(klyron_v8_context_t* ctx,
                                               const unsigned char* data,
                                               size_t length) {
    if (!ctx || !data || length == 0) return nullptr;
    auto iso = get_iso(ctx);
    v8::Locker locker(iso);
    v8::Isolate::Scope iso_scope(iso);
    v8::HandleScope scope(iso);

    auto store = v8::ArrayBuffer::NewBackingStore(
        const_cast<unsigned char*>(data), length,
        v8::BackingStore::EmptyDeleter, nullptr);
    auto ab = v8::ArrayBuffer::New(iso, std::move(store));
    return new klyron_v8_value(iso, ab, ctx->parent);
}
