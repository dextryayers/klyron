#include "klyron_v8.h"
#include "cpp/impl/internal.h"

#include <cstdio>
#include <cstring>
#include <ctime>
#include <string>

static std::string timestamp() {
    time_t now = time(nullptr);
    struct tm result;
    struct tm* t = localtime_r(&now, &result);
    char buf[64];
    strftime(buf, sizeof(buf), "%H:%M:%S", t);
    return std::string("[") + buf + "]";
}

static void console_output(const char* level, const char* msg, FILE* fp) {
    fprintf(fp, "%s %s: %s\n", timestamp().c_str(), level, msg);
    fflush(fp);
}

klyron_v8_value_t* klyron_v8_console_new(klyron_v8_context_t* ctx) {
    if (!ctx) return nullptr;
    auto iso = get_iso(ctx);
    v8::Locker locker(iso);
    v8::Isolate::Scope iso_scope(iso);
    v8::HandleScope scope(iso);
    auto context = get_ctx(ctx);
    v8::Context::Scope ctx_scope(context);

    auto obj = v8::Object::New(iso);

    auto log_fn = v8::Function::New(context,
        [](const v8::FunctionCallbackInfo<v8::Value>& info) {
            auto iso = info.GetIsolate();
            v8::HandleScope scope(iso);
            v8::String::Utf8Value str(iso, info[0]);
            if (*str) console_output("LOG", *str, stdout);
        }).ToLocalChecked();
    obj->Set(context, v8::String::NewFromUtf8(iso, "log", v8::NewStringType::kNormal).ToLocalChecked(), log_fn).Check();

    auto warn_fn = v8::Function::New(context,
        [](const v8::FunctionCallbackInfo<v8::Value>& info) {
            auto iso = info.GetIsolate();
            v8::HandleScope scope(iso);
            v8::String::Utf8Value str(iso, info[0]);
            if (*str) console_output("WARN", *str, stderr);
        }).ToLocalChecked();
    obj->Set(context, v8::String::NewFromUtf8(iso, "warn", v8::NewStringType::kNormal).ToLocalChecked(), warn_fn).Check();

    auto error_fn = v8::Function::New(context,
        [](const v8::FunctionCallbackInfo<v8::Value>& info) {
            auto iso = info.GetIsolate();
            v8::HandleScope scope(iso);
            v8::String::Utf8Value str(iso, info[0]);
            if (*str) console_output("ERROR", *str, stderr);
        }).ToLocalChecked();
    obj->Set(context, v8::String::NewFromUtf8(iso, "error", v8::NewStringType::kNormal).ToLocalChecked(), error_fn).Check();

    auto info_fn = v8::Function::New(context,
        [](const v8::FunctionCallbackInfo<v8::Value>& info) {
            auto iso = info.GetIsolate();
            v8::HandleScope scope(iso);
            v8::String::Utf8Value str(iso, info[0]);
            if (*str) console_output("INFO", *str, stdout);
        }).ToLocalChecked();
    obj->Set(context, v8::String::NewFromUtf8(iso, "info", v8::NewStringType::kNormal).ToLocalChecked(), info_fn).Check();

    auto debug_fn = v8::Function::New(context,
        [](const v8::FunctionCallbackInfo<v8::Value>& info) {
            auto iso = info.GetIsolate();
            v8::HandleScope scope(iso);
            v8::String::Utf8Value str(iso, info[0]);
            if (*str) console_output("DEBUG", *str, stdout);
        }).ToLocalChecked();
    obj->Set(context, v8::String::NewFromUtf8(iso, "debug", v8::NewStringType::kNormal).ToLocalChecked(), debug_fn).Check();

    return new klyron_v8_value(iso, obj, ctx->parent);
}

void klyron_v8_console_log(klyron_v8_context_t* ctx, const char* msg) {
    if (msg) console_output("LOG", msg, stdout);
}

void klyron_v8_console_warn(klyron_v8_context_t* ctx, const char* msg) {
    if (msg) console_output("WARN", msg, stderr);
}

void klyron_v8_console_error(klyron_v8_context_t* ctx, const char* msg) {
    if (msg) console_output("ERROR", msg, stderr);
}

void klyron_v8_console_info(klyron_v8_context_t* ctx, const char* msg) {
    if (msg) console_output("INFO", msg, stdout);
}

void klyron_v8_console_debug(klyron_v8_context_t* ctx, const char* msg) {
    if (msg) console_output("DEBUG", msg, stdout);
}
