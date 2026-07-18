#include "klyron_jsc.h"
#include "cpp/impl/internal.h"
#include <cstring>
#include <cstdlib>
#include <vector>

static const char b64_table[] = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
static const char b64url_table[] = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

static std::string base64_encode(const unsigned char* data, size_t len, bool url) {
    const char* table = url ? b64url_table : b64_table;
    size_t out_len = ((len + 2) / 3) * 4;
    std::string out(out_len, '=');
    size_t i = 0, j = 0;
    while (i < len) {
        unsigned a = data[i++];
        unsigned b = i < len ? data[i++] : 0;
        unsigned c = i < len ? data[i++] : 0;
        unsigned triplet = (a << 16) | (b << 8) | c;
        out[j++] = table[(triplet >> 18) & 0x3f];
        out[j++] = table[(triplet >> 12) & 0x3f];
        if (j < out_len) out[j] = table[(triplet >> 6) & 0x3f];
        if (j + 1 < out_len) out[j + 1] = table[triplet & 0x3f];
        j += 2;
    }
    return out;
}

static int b64_char_val(char c) {
    if (c >= 'A' && c <= 'Z') return c - 'A';
    if (c >= 'a' && c <= 'z') return c - 'a' + 26;
    if (c >= '0' && c <= '9') return c - '0' + 52;
    if (c == '+' || c == '-') return 62;
    if (c == '/' || c == '_') return 63;
    return -1;
}

static std::vector<unsigned char> base64_decode(const char* str, size_t len) {
    std::vector<unsigned char> out;
    out.reserve((len / 4) * 3);
    int buf = 0, bits = 0;
    for (size_t i = 0; i < len; i++) {
        if (str[i] == '=') break;
        int v = b64_char_val(str[i]);
        if (v < 0) continue;
        buf = (buf << 6) | v;
        bits += 6;
        if (bits >= 8) {
            bits -= 8;
            out.push_back((unsigned char)(buf >> bits));
            buf &= (1 << bits) - 1;
        }
    }
    return out;
}

static std::string hex_encode(const unsigned char* data, size_t len) {
    static const char hex[] = "0123456789abcdef";
    std::string out(len * 2, '\0');
    for (size_t i = 0; i < len; i++) {
        out[i*2] = hex[(data[i] >> 4) & 0xf];
        out[i*2+1] = hex[data[i] & 0xf];
    }
    return out;
}

static std::vector<unsigned char> hex_decode(const char* str, size_t len) {
    std::vector<unsigned char> out;
    out.reserve(len / 2);
    auto hex_val = [](char c) -> int {
        if (c >= '0' && c <= '9') return c - '0';
        if (c >= 'a' && c <= 'f') return c - 'a' + 10;
        if (c >= 'A' && c <= 'F') return c - 'A' + 10;
        return -1;
    };
    for (size_t i = 0; i + 1 < len; i += 2) {
        int hi = hex_val(str[i]);
        int lo = hex_val(str[i+1]);
        if (hi < 0 || lo < 0) break;
        out.push_back((unsigned char)((hi << 4) | lo));
    }
    return out;
}

klyron_jsc_string_result_t klyron_jsc_encode_text(klyron_jsc_engine_t* engine, const char* input) {
    klyron_jsc_string_result_t result = {false, nullptr, 0, {0}};
    if (!engine || !input) return result;
    jsc_set_string_result(&result, std::string(input));
    return result;
}

klyron_jsc_value_t* klyron_jsc_encode_into(klyron_jsc_engine_t* engine, const char* input, klyron_jsc_value_t* dst, size_t dst_offset) {
    if (!engine || !input || !dst) return nullptr;
    size_t len = std::strlen(input);
    if (!JSValueIsObject(engine->ctx, dst->value)) return nullptr;
    JSObjectRef obj = (JSObjectRef)dst->value;
    JSValueRef exc = nullptr;
    JSTypedArrayType ta = JSValueGetTypedArrayType(engine->ctx, obj, &exc);
    if (exc || ta == kJSTypedArrayTypeNone) return nullptr;
    size_t arr_len = JSObjectGetTypedArrayLength(engine->ctx, obj, &exc);
    if (exc || dst_offset + len > arr_len) return nullptr;
    JSObjectRef buf = JSObjectGetTypedArrayBuffer(engine->ctx, obj, &exc);
    if (exc || !buf) return nullptr;
    void* ptr = JSObjectGetArrayBufferBytesPtr(engine->ctx, buf, &exc);
    if (exc || !ptr) return nullptr;
    std::memcpy((unsigned char*)ptr + dst_offset, input, len);
    return dst;
}

klyron_jsc_value_t* klyron_jsc_decode_text(klyron_jsc_engine_t* engine, klyron_jsc_value_t* buf, const char* encoding) {
    if (!engine || !buf) return nullptr;
    if (!JSValueIsObject(engine->ctx, buf->value)) return nullptr;
    JSObjectRef obj = (JSObjectRef)buf->value;
    JSValueRef exc = nullptr;
    unsigned char* data = nullptr;
    size_t length = 0;
    JSTypedArrayType ta = JSValueGetTypedArrayType(engine->ctx, obj, &exc);
    if (exc) { jsc_capture_exception(engine, exc); return nullptr; }
    if (ta != kJSTypedArrayTypeNone) {
        JSObjectRef ab = JSObjectGetTypedArrayBuffer(engine->ctx, obj, &exc);
        if (exc || !ab) return nullptr;
        length = JSObjectGetArrayBufferByteLength(engine->ctx, ab, &exc);
        if (exc) return nullptr;
        data = (unsigned char*)JSObjectGetArrayBufferBytesPtr(engine->ctx, ab, &exc);
    } else {
        length = JSObjectGetArrayBufferByteLength(engine->ctx, obj, &exc);
        if (exc) return nullptr;
        data = (unsigned char*)JSObjectGetArrayBufferBytesPtr(engine->ctx, obj, &exc);
    }
    if (exc || !data) return nullptr;
    std::string result_str;
    if (!encoding || std::strcmp(encoding, "utf8") == 0 || std::strcmp(encoding, "utf-8") == 0) {
        result_str.assign((const char*)data, length);
    } else if (std::strcmp(encoding, "hex") == 0) {
        result_str = hex_encode(data, length);
    } else if (std::strcmp(encoding, "base64") == 0) {
        result_str = base64_encode(data, length, false);
    } else if (std::strcmp(encoding, "base64url") == 0) {
        result_str = base64_encode(data, length, true);
    } else {
        result_str.assign((const char*)data, length);
    }
    JSStringRef jsstr = jsc_string_from_cstr(result_str.c_str());
    if (!jsstr) return nullptr;
    JSValueRef val = JSValueMakeString(engine->ctx, jsstr);
    JSStringRelease(jsstr);
    auto v = new klyron_jsc_value_t(engine->ctx, val);
    v->protect();
    return v;
}

klyron_jsc_string_result_t klyron_jsc_base64_encode(klyron_jsc_engine_t* engine, const char* input) {
    klyron_jsc_string_result_t result = {false, nullptr, 0, {0}};
    if (!engine || !input) return result;
    std::string enc = base64_encode((const unsigned char*)input, std::strlen(input), false);
    jsc_set_string_result(&result, enc);
    return result;
}

klyron_jsc_value_t* klyron_jsc_base64_decode(klyron_jsc_engine_t* engine, const char* input) {
    if (!engine || !input) return nullptr;
    auto bytes = base64_decode(input, std::strlen(input));
    if (bytes.empty()) {
        JSValueRef val = JSValueMakeString(engine->ctx, jsc_string_from_cstr(""));
        auto v = new klyron_jsc_value_t(engine->ctx, val);
        v->protect();
        return v;
    }
    void* mem = std::malloc(bytes.size());
    if (!mem) return nullptr;
    std::memcpy(mem, bytes.data(), bytes.size());
    JSValueRef exc = nullptr;
    JSValueRef ab = JSObjectMakeArrayBufferWithBytesNoCopy(
        engine->ctx, mem, bytes.size(),
        [](void* p, void* c) { std::free(p); }, nullptr, &exc);
    if (exc || !ab) {
        std::free(mem);
        jsc_capture_exception(engine, exc);
        return nullptr;
    }
    auto v = new klyron_jsc_value_t(engine->ctx, ab);
    v->protect();
    return v;
}

klyron_jsc_string_result_t klyron_jsc_hex_encode(klyron_jsc_engine_t* engine, const char* input) {
    klyron_jsc_string_result_t result = {false, nullptr, 0, {0}};
    if (!engine || !input) return result;
    std::string enc = hex_encode((const unsigned char*)input, std::strlen(input));
    jsc_set_string_result(&result, enc);
    return result;
}

klyron_jsc_value_t* klyron_jsc_hex_decode(klyron_jsc_engine_t* engine, const char* input) {
    if (!engine || !input) return nullptr;
    auto bytes = hex_decode(input, std::strlen(input));
    void* mem = std::malloc(bytes.size());
    if (!mem) return nullptr;
    std::memcpy(mem, bytes.data(), bytes.size());
    JSValueRef exc = nullptr;
    JSValueRef ab = JSObjectMakeArrayBufferWithBytesNoCopy(
        engine->ctx, mem, bytes.size(),
        [](void* p, void* c) { std::free(p); }, nullptr, &exc);
    if (exc || !ab) {
        std::free(mem);
        jsc_capture_exception(engine, exc);
        return nullptr;
    }
    auto v = new klyron_jsc_value_t(engine->ctx, ab);
    v->protect();
    return v;
}
