#include "klyron_v8.h"
#include "cpp/impl/internal.h"

#include <cstring>
#include <cstdlib>
#include <algorithm>

klyron_v8_value_t* klyron_v8_buffer_new(klyron_v8_context_t* ctx, size_t size) {
    if (!ctx) return nullptr;
    auto iso = get_iso(ctx);
    v8::Locker locker(iso);
    v8::Isolate::Scope iso_scope(iso);
    v8::HandleScope scope(iso);
    auto context = get_ctx(ctx);
    v8::Context::Scope ctx_scope(context);

    auto ab = v8::ArrayBuffer::New(iso, size);
    auto ta = v8::Uint8Array::New(ab, 0, size);
    return new klyron_v8_value(iso, ta, ctx->parent);
}

klyron_v8_value_t* klyron_v8_buffer_from_string(klyron_v8_context_t* ctx, const char* str, const char* encoding) {
    if (!ctx || !str) return nullptr;
    auto iso = get_iso(ctx);
    v8::Locker locker(iso);
    v8::Isolate::Scope iso_scope(iso);
    v8::HandleScope scope(iso);
    auto context = get_ctx(ctx);
    v8::Context::Scope ctx_scope(context);

    size_t len = std::strlen(str);
    std::string enc = encoding ? encoding : "utf8";

    auto ab = v8::ArrayBuffer::New(iso, len);
    std::memcpy(ab->GetBackingStore()->Data(), str, len);
    auto ta = v8::Uint8Array::New(ab, 0, len);
    return new klyron_v8_value(iso, ta, ctx->parent);
}

klyron_v8_value_t* klyron_v8_buffer_from_bytes(klyron_v8_context_t* ctx, const unsigned char* data, size_t length) {
    if (!ctx || !data) return nullptr;
    auto iso = get_iso(ctx);
    v8::Locker locker(iso);
    v8::Isolate::Scope iso_scope(iso);
    v8::HandleScope scope(iso);
    auto context = get_ctx(ctx);
    v8::Context::Scope ctx_scope(context);

    auto ab = v8::ArrayBuffer::New(iso, length);
    std::memcpy(ab->GetBackingStore()->Data(), data, length);
    auto ta = v8::Uint8Array::New(ab, 0, length);
    return new klyron_v8_value(iso, ta, ctx->parent);
}

klyron_v8_string_result_t klyron_v8_buffer_to_string(klyron_v8_context_t* ctx, klyron_v8_value_t* buf, const char* encoding, size_t start, size_t end) {
    klyron_v8_string_result_t result = {false, nullptr, 0, {0}};
    if (!ctx || !buf) { set_error(&result, "null context or buffer"); return result; }

    auto iso = get_iso(ctx);
    v8::Locker locker(iso);
    v8::Isolate::Scope iso_scope(iso);
    v8::HandleScope scope(iso);
    auto context = get_ctx(ctx);
    v8::Context::Scope ctx_scope(context);

    auto val = buf->value->Get(iso);
    if (!val->IsTypedArray() && !val->IsArrayBuffer()) {
        set_error(&result, "not a buffer");
        return result;
    }

    v8::Local<v8::ArrayBuffer> ab;
    size_t offset = 0;
    size_t total = 0;

    if (val->IsTypedArray()) {
        auto ta = val.As<v8::TypedArray>();
        ab = ta->Buffer();
        offset = ta->ByteOffset();
        total = ta->ByteLength();
    } else {
        ab = val.As<v8::ArrayBuffer>();
        total = ab->ByteLength();
    }

    size_t s = std::min(start, total);
    size_t e = std::min(end, total);
    if (e <= s) { set_result(&result, ""); return result; }

    auto store = ab->GetBackingStore();
    auto raw = static_cast<const char*>(store->Data()) + offset + s;
    size_t copy_len = e - s;
    std::string str(raw, copy_len);
    set_result(&result, str);
    return result;
}

unsigned char* klyron_v8_buffer_get_data(klyron_v8_context_t* ctx, klyron_v8_value_t* buf) {
    if (!ctx || !buf) return nullptr;
    auto iso = get_iso(ctx);
    v8::Locker locker(iso);
    v8::Isolate::Scope iso_scope(iso);
    v8::HandleScope scope(iso);
    auto context = get_ctx(ctx);
    v8::Context::Scope ctx_scope(context);

    auto val = buf->value->Get(iso);
    if (val->IsTypedArray()) {
        auto ta = val.As<v8::TypedArray>();
        auto ab = ta->Buffer();
        auto store = ab->GetBackingStore();
        return static_cast<unsigned char*>(store->Data()) + ta->ByteOffset();
    }
    if (val->IsArrayBuffer()) {
        auto ab = val.As<v8::ArrayBuffer>();
        return static_cast<unsigned char*>(ab->GetBackingStore()->Data());
    }
    return nullptr;
}

size_t klyron_v8_buffer_get_length(klyron_v8_context_t* ctx, klyron_v8_value_t* buf) {
    if (!ctx || !buf) return 0;
    auto iso = get_iso(ctx);
    v8::Locker locker(iso);
    v8::Isolate::Scope iso_scope(iso);
    v8::HandleScope scope(iso);
    auto val = buf->value->Get(iso);
    if (val->IsTypedArray()) return val.As<v8::TypedArray>()->ByteLength();
    if (val->IsArrayBuffer()) return val.As<v8::ArrayBuffer>()->ByteLength();
    return 0;
}

klyron_v8_result_t klyron_v8_buffer_copy(klyron_v8_context_t* ctx, klyron_v8_value_t* dst, size_t dst_offset, klyron_v8_value_t* src, size_t src_offset, size_t count) {
    klyron_v8_result_t result = {false, {0}};
    if (!ctx || !dst || !src) return result;

    auto iso = get_iso(ctx);
    v8::Locker locker(iso);
    v8::Isolate::Scope iso_scope(iso);
    v8::HandleScope scope(iso);

    auto dval = dst->value->Get(iso);
    auto sval = src->value->Get(iso);
    if (!dval->IsTypedArray() || !sval->IsTypedArray()) return result;

    auto dta = dval.As<v8::TypedArray>();
    auto sta = sval.As<v8::TypedArray>();
    auto db = dta->Buffer();
    auto sb = sta->Buffer();
    auto dstore = db->GetBackingStore();
    auto sstore = sb->GetBackingStore();

    size_t dlen = dta->ByteLength();
    size_t slen = sta->ByteLength();
    size_t to_copy = std::min({count, dlen - dst_offset, slen - src_offset});

    auto dptr = static_cast<char*>(dstore->Data()) + dta->ByteOffset() + dst_offset;
    auto sptr = static_cast<const char*>(sstore->Data()) + sta->ByteOffset() + src_offset;
    std::memmove(dptr, sptr, to_copy);

    set_bool_result(&result, true);
    return result;
}

klyron_v8_value_t* klyron_v8_buffer_concat(klyron_v8_context_t* ctx, klyron_v8_value_t** bufs, size_t count) {
    if (!ctx || !bufs || count == 0) return nullptr;

    auto iso = get_iso(ctx);
    v8::Locker locker(iso);
    v8::Isolate::Scope iso_scope(iso);
    v8::HandleScope scope(iso);
    auto context = get_ctx(ctx);
    v8::Context::Scope ctx_scope(context);

    size_t total = 0;
    for (size_t i = 0; i < count; i++) {
        if (bufs[i]) total += klyron_v8_buffer_get_length(ctx, bufs[i]);
    }

    auto ab = v8::ArrayBuffer::New(iso, total);
    auto ptr = static_cast<char*>(ab->GetBackingStore()->Data());
    size_t offset = 0;
    for (size_t i = 0; i < count; i++) {
        if (!bufs[i]) continue;
        auto val = bufs[i]->value->Get(iso);
        if (val->IsTypedArray()) {
            auto ta = val.As<v8::TypedArray>();
            auto src_ab = ta->Buffer();
            auto src_store = src_ab->GetBackingStore();
            size_t len = ta->ByteLength();
            std::memcpy(ptr + offset, static_cast<char*>(src_store->Data()) + ta->ByteOffset(), len);
            offset += len;
        }
    }

    auto ta = v8::Uint8Array::New(ab, 0, total);
    return new klyron_v8_value(iso, ta, ctx->parent);
}

klyron_v8_value_t* klyron_v8_buffer_slice(klyron_v8_context_t* ctx, klyron_v8_value_t* buf, size_t start, size_t end) {
    if (!ctx || !buf) return nullptr;
    auto iso = get_iso(ctx);
    v8::Locker locker(iso);
    v8::Isolate::Scope iso_scope(iso);
    v8::HandleScope scope(iso);
    auto context = get_ctx(ctx);
    v8::Context::Scope ctx_scope(context);

    auto val = buf->value->Get(iso);
    if (!val->IsTypedArray()) return nullptr;

    auto ta = val.As<v8::TypedArray>();
    auto src_ab = ta->Buffer();
    auto store = src_ab->GetBackingStore();
    size_t total = ta->ByteLength();
    size_t s = std::min(start, total);
    size_t e = std::min(end, total);
    size_t len = (e > s) ? (e - s) : 0;

    auto dst_ab = v8::ArrayBuffer::New(iso, len);
    if (len > 0) {
        std::memcpy(dst_ab->GetBackingStore()->Data(),
                    static_cast<char*>(store->Data()) + ta->ByteOffset() + s, len);
    }
    auto dst_ta = v8::Uint8Array::New(dst_ab, 0, len);
    return new klyron_v8_value(iso, dst_ta, ctx->parent);
}

klyron_v8_result_t klyron_v8_buffer_write(klyron_v8_context_t* ctx, klyron_v8_value_t* buf, const unsigned char* data, size_t offset, size_t length) {
    klyron_v8_result_t result = {false, {0}};
    if (!ctx || !buf || !data) return result;

    auto iso = get_iso(ctx);
    v8::Locker locker(iso);
    v8::Isolate::Scope iso_scope(iso);
    v8::HandleScope scope(iso);

    auto val = buf->value->Get(iso);
    if (!val->IsTypedArray()) return result;

    auto ta = val.As<v8::TypedArray>();
    size_t total = ta->ByteLength();
    if (offset + length > total) return result;

    auto ab = ta->Buffer();
    auto store = ab->GetBackingStore();
    std::memcpy(static_cast<char*>(store->Data()) + ta->ByteOffset() + offset, data, length);
    set_bool_result(&result, true);
    return result;
}
