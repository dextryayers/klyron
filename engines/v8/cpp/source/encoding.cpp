#include "klyron_v8.h"
#include "cpp/impl/internal.h"

#include <cstring>
#include <string>
#include <vector>
#include <algorithm>

static const char base64_chars[] =
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

static std::string base64_encode_internal(const unsigned char* data, size_t len) {
    std::string result;
    result.reserve(((len + 2) / 3) * 4);
    for (size_t i = 0; i < len; i += 3) {
        unsigned char b0 = data[i];
        unsigned char b1 = (i + 1 < len) ? data[i + 1] : 0;
        unsigned char b2 = (i + 2 < len) ? data[i + 2] : 0;
        result += base64_chars[b0 >> 2];
        result += base64_chars[((b0 & 0x03) << 4) | (b1 >> 4)];
        result += (i + 1 < len) ? base64_chars[((b1 & 0x0f) << 2) | (b2 >> 6)] : '=';
        result += (i + 2 < len) ? base64_chars[b2 & 0x3f] : '=';
    }
    return result;
}

static std::vector<unsigned char> base64_decode_internal(const char* input) {
    std::vector<unsigned char> result;
    size_t len = std::strlen(input);
    if (len == 0) return result;

    std::string clean;
    for (size_t i = 0; i < len; i++) {
        if (input[i] != '=' && input[i] != '\n' && input[i] != '\r' && input[i] != ' ') {
            clean += input[i];
        }
    }

    static unsigned char decode_table[256] = {0};
    static bool table_init = false;
    if (!table_init) {
        for (int i = 0; i < 64; i++) {
            decode_table[static_cast<unsigned char>(base64_chars[i])] = i;
        }
        table_init = true;
    }

    result.reserve((clean.size() / 4) * 3);
    for (size_t i = 0; i + 3 < clean.size(); i += 4) {
        unsigned char c0 = decode_table[static_cast<unsigned char>(clean[i])];
        unsigned char c1 = decode_table[static_cast<unsigned char>(clean[i + 1])];
        unsigned char c2 = decode_table[static_cast<unsigned char>(clean[i + 2])];
        unsigned char c3 = decode_table[static_cast<unsigned char>(clean[i + 3])];
        result.push_back((c0 << 2) | (c1 >> 4));
        result.push_back((c1 << 4) | (c2 >> 2));
        result.push_back((c2 << 6) | c3);
    }
    return result;
}

static std::string hex_encode_internal(const unsigned char* data, size_t len) {
    static const char hex_chars[] = "0123456789abcdef";
    std::string result;
    result.reserve(len * 2);
    for (size_t i = 0; i < len; i++) {
        result += hex_chars[data[i] >> 4];
        result += hex_chars[data[i] & 0x0f];
    }
    return result;
}

static std::vector<unsigned char> hex_decode_internal(const char* input) {
    std::vector<unsigned char> result;
    size_t len = std::strlen(input);
    if (len < 2) return result;
    result.reserve(len / 2);
    for (size_t i = 0; i + 1 < len; i += 2) {
        auto h2c = [](char c) -> unsigned char {
            if (c >= '0' && c <= '9') return c - '0';
            if (c >= 'a' && c <= 'f') return c - 'a' + 10;
            if (c >= 'A' && c <= 'F') return c - 'A' + 10;
            return 0;
        };
        result.push_back((h2c(input[i]) << 4) | h2c(input[i + 1]));
    }
    return result;
}

klyron_v8_value_t* klyron_v8_encoding_text_encoder_new(klyron_v8_context_t* ctx) {
    if (!ctx) return nullptr;
    auto iso = get_iso(ctx);
    v8::Locker locker(iso);
    v8::Isolate::Scope iso_scope(iso);
    v8::HandleScope scope(iso);
    auto context = get_ctx(ctx);
    v8::Context::Scope ctx_scope(context);

    auto obj = v8::Object::New(iso);
    auto encoding = v8::String::NewFromUtf8(iso, "utf-8", v8::NewStringType::kNormal).ToLocalChecked();
    obj->Set(context, v8::String::NewFromUtf8(iso, "encoding", v8::NewStringType::kNormal).ToLocalChecked(), encoding).Check();

    auto encode_fn = v8::Function::New(context,
        [](const v8::FunctionCallbackInfo<v8::Value>& info) {
            auto iso = info.GetIsolate();
            v8::HandleScope scope(iso);
            v8::String::Utf8Value input(iso, info[0]);
            if (!*input) { info.GetReturnValue().Set(v8::Uint8Array::New(v8::ArrayBuffer::New(iso, 0), 0, 0)); return; }
            size_t len = input.length();
            auto ab = v8::ArrayBuffer::New(iso, len);
            std::memcpy(ab->GetBackingStore()->Data(), *input, len);
            info.GetReturnValue().Set(v8::Uint8Array::New(ab, 0, len));
        }).ToLocalChecked();
    obj->Set(context, v8::String::NewFromUtf8(iso, "encode", v8::NewStringType::kNormal).ToLocalChecked(), encode_fn).Check();

    return new klyron_v8_value(iso, obj, ctx->parent);
}

klyron_v8_value_t* klyron_v8_encoding_text_decoder_new(klyron_v8_context_t* ctx, const char* label) {
    if (!ctx) return nullptr;
    auto iso = get_iso(ctx);
    v8::Locker locker(iso);
    v8::Isolate::Scope iso_scope(iso);
    v8::HandleScope scope(iso);
    auto context = get_ctx(ctx);
    v8::Context::Scope ctx_scope(context);

    std::string enc = label ? label : "utf-8";

    auto obj = v8::Object::New(iso);
    auto encoding = v8::String::NewFromUtf8(iso, enc.c_str(), v8::NewStringType::kNormal).ToLocalChecked();
    obj->Set(context, v8::String::NewFromUtf8(iso, "encoding", v8::NewStringType::kNormal).ToLocalChecked(), encoding).Check();

    auto decode_fn = v8::Function::New(context,
        [](const v8::FunctionCallbackInfo<v8::Value>& info) {
            auto iso = info.GetIsolate();
            v8::HandleScope scope(iso);
            if (info[0]->IsArrayBuffer() || info[0]->IsTypedArray()) {
                v8::Local<v8::ArrayBuffer> ab;
                size_t offset = 0;
                size_t len = 0;
                if (info[0]->IsTypedArray()) {
                    auto ta = info[0].As<v8::TypedArray>();
                    ab = ta->Buffer();
                    offset = ta->ByteOffset();
                    len = ta->ByteLength();
                } else {
                    ab = info[0].As<v8::ArrayBuffer>();
                    len = ab->ByteLength();
                }
                auto store = ab->GetBackingStore();
                auto ptr = static_cast<const char*>(store->Data()) + offset;
                auto str = v8::String::NewFromUtf8(iso, ptr, v8::NewStringType::kNormal, (int)len).ToLocalChecked();
                info.GetReturnValue().Set(str);
            }
        }).ToLocalChecked();
    obj->Set(context, v8::String::NewFromUtf8(iso, "decode", v8::NewStringType::kNormal).ToLocalChecked(), decode_fn).Check();

    return new klyron_v8_value(iso, obj, ctx->parent);
}

klyron_v8_value_t* klyron_v8_encoding_encode(klyron_v8_context_t* ctx, const char* input) {
    if (!ctx || !input) return nullptr;
    auto iso = get_iso(ctx);
    v8::Locker locker(iso);
    v8::Isolate::Scope iso_scope(iso);
    v8::HandleScope scope(iso);
    auto context = get_ctx(ctx);
    v8::Context::Scope ctx_scope(context);

    size_t len = std::strlen(input);
    auto ab = v8::ArrayBuffer::New(iso, len);
    std::memcpy(ab->GetBackingStore()->Data(), input, len);
    auto ta = v8::Uint8Array::New(ab, 0, len);
    return new klyron_v8_value(iso, ta, ctx->parent);
}

klyron_v8_string_result_t klyron_v8_encoding_decode(klyron_v8_context_t* ctx, const unsigned char* data, size_t length, const char* encoding) {
    klyron_v8_string_result_t result = {false, nullptr, 0, {0}};
    if (!data || length == 0) { set_result(&result, ""); return result; }
    std::string str(reinterpret_cast<const char*>(data), length);
    set_result(&result, str);
    return result;
}

klyron_v8_string_result_t klyron_v8_encoding_base64_encode(klyron_v8_context_t* ctx, const unsigned char* data, size_t length) {
    klyron_v8_string_result_t result = {false, nullptr, 0, {0}};
    if (!data || length == 0) { set_result(&result, ""); return result; }
    set_result(&result, base64_encode_internal(data, length));
    return result;
}

klyron_v8_value_t* klyron_v8_encoding_base64_decode(klyron_v8_context_t* ctx, const char* input) {
    if (!ctx || !input) return nullptr;
    auto decoded = base64_decode_internal(input);
    auto iso = get_iso(ctx);
    v8::Locker locker(iso);
    v8::Isolate::Scope iso_scope(iso);
    v8::HandleScope scope(iso);
    auto context = get_ctx(ctx);
    v8::Context::Scope ctx_scope(context);

    auto ab = v8::ArrayBuffer::New(iso, decoded.size());
    if (!decoded.empty()) std::memcpy(ab->GetBackingStore()->Data(), decoded.data(), decoded.size());
    auto ta = v8::Uint8Array::New(ab, 0, decoded.size());
    return new klyron_v8_value(iso, ta, ctx->parent);
}

klyron_v8_string_result_t klyron_v8_encoding_hex_encode(klyron_v8_context_t* ctx, const unsigned char* data, size_t length) {
    klyron_v8_string_result_t result = {false, nullptr, 0, {0}};
    if (!data || length == 0) { set_result(&result, ""); return result; }
    set_result(&result, hex_encode_internal(data, length));
    return result;
}

klyron_v8_value_t* klyron_v8_encoding_hex_decode(klyron_v8_context_t* ctx, const char* input) {
    if (!ctx || !input) return nullptr;
    auto decoded = hex_decode_internal(input);
    auto iso = get_iso(ctx);
    v8::Locker locker(iso);
    v8::Isolate::Scope iso_scope(iso);
    v8::HandleScope scope(iso);
    auto context = get_ctx(ctx);
    v8::Context::Scope ctx_scope(context);

    auto ab = v8::ArrayBuffer::New(iso, decoded.size());
    if (!decoded.empty()) std::memcpy(ab->GetBackingStore()->Data(), decoded.data(), decoded.size());
    auto ta = v8::Uint8Array::New(ab, 0, decoded.size());
    return new klyron_v8_value(iso, ta, ctx->parent);
}
