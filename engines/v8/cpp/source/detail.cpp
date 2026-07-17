#include "klyron_v8.h"
#include "cpp/impl/internal.h"

#include <cstring>

v8::Isolate* get_iso(klyron_v8_context_t* ctx) {
    return ctx && ctx->parent ? ctx->parent->isolate : nullptr;
}

v8::MaybeLocal<v8::Context> get_ctx_maybe(klyron_v8_context_t* ctx) {
    if (!ctx || !ctx->context || !ctx->parent || !ctx->parent->isolate) {
        return v8::MaybeLocal<v8::Context>();
    }
    return ctx->context->Get(ctx->parent->isolate);
}

v8::Local<v8::Context> get_ctx(klyron_v8_context_t* ctx) {
    auto maybe = get_ctx_maybe(ctx);
    v8::Local<v8::Context> local;
    if (maybe.ToLocal(&local)) {
        return local;
    }
    return v8::Local<v8::Context>();
}

void set_error(klyron_v8_string_result_t* r, const char* msg) {
    if (!r) return;
    r->success = false;
    r->data = nullptr;
    r->length = 0;
    std::strncpy(r->error, msg, KLYRON_V8_ERROR_BUF_SIZE - 1);
    r->error[KLYRON_V8_ERROR_BUF_SIZE - 1] = '\0';
}

void set_result(klyron_v8_string_result_t* r, const std::string& s) {
    if (!r) return;
    r->success = true;
    r->length = s.size();
    r->data = static_cast<char*>(std::malloc(s.size() + 1));
    if (r->data) {
        std::memcpy(r->data, s.data(), s.size());
        r->data[s.size()] = '\0';
    }
    r->error[0] = '\0';
}

void set_bool_result(klyron_v8_result_t* r, bool ok) {
    if (!r) return;
    r->success = ok;
    if (!ok) {
        std::strncpy(r->error, "operation failed", KLYRON_V8_ERROR_BUF_SIZE - 1);
        r->error[KLYRON_V8_ERROR_BUF_SIZE - 1] = '\0';
    } else {
        r->error[0] = '\0';
    }
}

void capture_exception(klyron_v8_context_t* ctx, v8::TryCatch& tc,
                       char* buf, size_t buf_size) {
    if (!buf || buf_size == 0) return;
    if (!tc.HasCaught()) {
        buf[0] = '\0';
        return;
    }
    auto iso = get_iso(ctx);
    if (!iso) {
        std::strncpy(buf, "no isolate", buf_size - 1);
        buf[buf_size - 1] = '\0';
        return;
    }
    v8::HandleScope scope(iso);
    auto exc = tc.Exception();
    if (exc.IsEmpty()) {
        std::strncpy(buf, "empty exception", buf_size - 1);
        buf[buf_size - 1] = '\0';
        return;
    }
    v8::String::Utf8Value msg(iso, exc);
    std::strncpy(buf, *msg ? *msg : "unknown error", buf_size - 1);
    buf[buf_size - 1] = '\0';
}
