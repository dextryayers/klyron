#include "klyron_jsc.h"
#include "cpp/impl/internal.h"
#include <cstring>
#include <cstdlib>
#include <algorithm>

static klyron_jsc_value_t* buffer_from_js(klyron_jsc_engine_t* engine, const unsigned char* data, size_t length) {
    if (!engine || !engine->ctx) return nullptr;
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

static bool buffer_get_bytes(klyron_jsc_engine_t* engine, klyron_jsc_value_t* value, unsigned char** out_data, size_t* out_len) {
    if (!engine || !value || !out_data || !out_len) return false;
    if (!JSValueIsObject(engine->ctx, value->value)) return false;
    JSObjectRef obj = (JSObjectRef)value->value;
    JSValueRef exc = nullptr;
    JSTypedArrayType ta = JSValueGetTypedArrayType(engine->ctx, obj, &exc);
    if (exc) return false;
    void* ptr = nullptr;
    size_t len = 0;
    if (ta == kJSTypedArrayTypeNone) {
        len = JSObjectGetArrayBufferByteLength(engine->ctx, obj, &exc);
        if (exc || len == 0) return false;
        ptr = JSObjectGetArrayBufferBytesPtr(engine->ctx, obj, &exc);
    } else {
        JSObjectRef buf = JSObjectGetTypedArrayBuffer(engine->ctx, obj, &exc);
        if (exc || !buf) return false;
        len = JSObjectGetArrayBufferByteLength(engine->ctx, buf, &exc);
        if (exc || len == 0) return false;
        ptr = JSObjectGetArrayBufferBytesPtr(engine->ctx, buf, &exc);
    }
    if (exc || !ptr || len == 0) return false;
    *out_len = len;
    *out_data = (unsigned char*)std::malloc(len);
    if (!*out_data) return false;
    std::memcpy(*out_data, ptr, len);
    return true;
}

klyron_jsc_value_t* klyron_jsc_buffer_alloc(klyron_jsc_engine_t* engine, size_t size) {
    if (!engine || size == 0) return nullptr;
    unsigned char* zeros = (unsigned char*)std::calloc(size, 1);
    if (!zeros) return nullptr;
    auto v = buffer_from_js(engine, zeros, size);
    std::free(zeros);
    return v;
}

klyron_jsc_value_t* klyron_jsc_buffer_from_string(klyron_jsc_engine_t* engine, const char* str) {
    if (!engine || !str) return nullptr;
    size_t len = std::strlen(str);
    return buffer_from_js(engine, (const unsigned char*)str, len);
}

klyron_jsc_value_t* klyron_jsc_buffer_concat(klyron_jsc_engine_t* engine, klyron_jsc_value_t** bufs, size_t count) {
    if (!engine || !bufs || count == 0) return engine ? klyron_jsc_buffer_alloc(engine, 0) : nullptr;
    size_t total = 0;
    for (size_t i = 0; i < count; i++) {
        unsigned char* d = nullptr; size_t l = 0;
        if (bufs[i] && buffer_get_bytes(engine, bufs[i], &d, &l)) {
            total += l;
            std::free(d);
        }
    }
    if (total == 0) return klyron_jsc_buffer_alloc(engine, 0);
    unsigned char* combined = (unsigned char*)std::malloc(total);
    if (!combined) return nullptr;
    size_t offset = 0;
    for (size_t i = 0; i < count; i++) {
        unsigned char* d = nullptr; size_t l = 0;
        if (bufs[i] && buffer_get_bytes(engine, bufs[i], &d, &l)) {
            std::memcpy(combined + offset, d, l);
            offset += l;
            std::free(d);
        }
    }
    auto v = buffer_from_js(engine, combined, total);
    std::free(combined);
    return v;
}

klyron_jsc_string_result_t klyron_jsc_buffer_to_string(klyron_jsc_engine_t* engine, klyron_jsc_value_t* buf, const char* encoding) {
    klyron_jsc_string_result_t result = {false, nullptr, 0, {0}};
    if (!engine || !buf) return result;
    unsigned char* data = nullptr; size_t len = 0;
    if (!buffer_get_bytes(engine, buf, &data, &len)) {
        jsc_set_error(engine, "buffer_to_string: cannot extract bytes");
        return result;
    }
    if (!encoding || std::strcmp(encoding, "utf8") == 0 || std::strcmp(encoding, "utf-8") == 0) {
        jsc_set_string_result(&result, std::string((const char*)data, len));
    } else if (std::strcmp(encoding, "hex") == 0) {
        static const char hex[] = "0123456789abcdef";
        std::string out(len * 2, '\0');
        for (size_t i = 0; i < len; i++) {
            out[i*2] = hex[(data[i] >> 4) & 0xf];
            out[i*2+1] = hex[data[i] & 0xf];
        }
        jsc_set_string_result(&result, out);
    } else if (std::strcmp(encoding, "base64") == 0) {
        static const char b64[] = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        size_t out_len = ((len + 2) / 3) * 4;
        std::string out(out_len, '\0');
        size_t i = 0, j = 0;
        while (i < len) {
            unsigned a = i < len ? data[i++] : 0;
            unsigned b = i < len ? data[i++] : 0;
            unsigned c = i < len ? data[i++] : 0;
            unsigned triplet = (a << 16) | (b << 8) | c;
            out[j++] = b64[(triplet >> 18) & 0x3f];
            out[j++] = b64[(triplet >> 12) & 0x3f];
            out[j++] = (j - 2 + 1 > out_len - 2) ? '=' : b64[(triplet >> 6) & 0x3f];
            out[j++] = (j - 1 + 1 > out_len - 1) ? '=' : b64[triplet & 0x3f];
        }
        jsc_set_string_result(&result, out);
    } else {
        jsc_set_string_result(&result, std::string((const char*)data, len));
    }
    std::free(data);
    return result;
}

klyron_jsc_value_t* klyron_jsc_buffer_slice(klyron_jsc_engine_t* engine, klyron_jsc_value_t* buf, size_t start, size_t end) {
    if (!engine || !buf) return nullptr;
    unsigned char* data = nullptr; size_t len = 0;
    if (!buffer_get_bytes(engine, buf, &data, &len)) return nullptr;
    start = std::min(start, len);
    end = std::min(end, len);
    size_t slice_len = end > start ? end - start : 0;
    auto v = buffer_from_js(engine, data + start, slice_len);
    std::free(data);
    return v;
}
