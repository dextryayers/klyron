#include "klyron_v8.h"
#include "cpp/impl/internal.h"

#include <cstring>

klyron_v8_script_t* klyron_v8_compile(klyron_v8_context_t* ctx,
                                      const char* source,
                                      const char* filename) {
    if (!ctx) return nullptr;
    auto iso = get_iso(ctx);
    if (!iso) return nullptr;

    v8::Locker locker(iso);
    v8::Isolate::Scope iso_scope(iso);
    v8::HandleScope scope(iso);
    auto context = get_ctx(ctx);
    v8::Context::Scope ctx_scope(context);

    v8::TryCatch tc(iso);

    auto source_str =
        v8::String::NewFromUtf8(iso, source, v8::NewStringType::kNormal)
            .ToLocalChecked();
    auto origin_str = filename
        ? v8::String::NewFromUtf8(iso, filename, v8::NewStringType::kNormal)
              .ToLocalChecked()
        : v8::String::NewFromUtf8(iso, "<eval>", v8::NewStringType::kNormal)
              .ToLocalChecked();

    v8::ScriptOrigin origin(origin_str);
    auto compiled = v8::Script::Compile(context, source_str, &origin);

    if (compiled.IsEmpty()) {
        capture_exception(ctx, tc, ctx->parent->error_buf,
                          KLYRON_V8_ERROR_BUF_SIZE);
        return nullptr;
    }

    return new klyron_v8_script(iso, compiled.ToLocalChecked(), ctx->parent);
}

klyron_v8_string_result_t klyron_v8_run(klyron_v8_context_t* ctx,
                                        klyron_v8_script_t* script) {
    klyron_v8_string_result_t result = {false, nullptr, 0, {0}};
    if (!ctx || !script) {
        set_error(&result, "null context or script");
        return result;
    }

    auto iso = get_iso(ctx);
    v8::Locker locker(iso);
    v8::Isolate::Scope iso_scope(iso);
    v8::HandleScope scope(iso);
    auto context = get_ctx(ctx);
    v8::Context::Scope ctx_scope(context);

    v8::TryCatch tc(iso);

    auto local_script = script->script->Get(iso);
    auto run_result = local_script->Run(context);

    if (run_result.IsEmpty()) {
        capture_exception(ctx, tc, result.error, KLYRON_V8_ERROR_BUF_SIZE);
        return result;
    }

    v8::String::Utf8Value utf8(iso, run_result.ToLocalChecked());
    if (*utf8) {
        set_result(&result, std::string(*utf8, utf8.length()));
    } else {
        set_result(&result, "undefined");
    }

    return result;
}

klyron_v8_string_result_t klyron_v8_eval(klyron_v8_context_t* ctx,
                                         const char* source,
                                         const char* filename) {
    auto script = klyron_v8_compile(ctx, source, filename);
    if (!script) {
        klyron_v8_string_result_t err_result = {false, nullptr, 0, {0}};
        return err_result;
    }

    auto result = klyron_v8_run(ctx, script);
    klyron_v8_script_dispose(script);
    return result;
}

void klyron_v8_script_dispose(klyron_v8_script_t* script) {
    delete script;
}
