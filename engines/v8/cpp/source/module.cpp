#include "klyron_v8.h"
#include "cpp/impl/internal.h"

#include <cstring>

klyron_v8_module_t* klyron_v8_module_compile(klyron_v8_context_t* ctx,
                                              const char* source,
                                              const char* origin) {
    if (!ctx) return nullptr;
    auto iso = get_iso(ctx);
    v8::Locker locker(iso);
    v8::Isolate::Scope iso_scope(iso);
    v8::HandleScope scope(iso);
    auto context = get_ctx(ctx);
    v8::Context::Scope ctx_scope(context);

    v8::TryCatch tc(iso);

    auto src =
        v8::String::NewFromUtf8(iso, source, v8::NewStringType::kNormal)
            .ToLocalChecked();
    auto name = origin
        ? v8::String::NewFromUtf8(iso, origin, v8::NewStringType::kNormal)
              .ToLocalChecked()
        : v8::String::NewFromUtf8(iso, "<module>", v8::NewStringType::kNormal)
              .ToLocalChecked();

    v8::ScriptOrigin script_origin(name);
    v8::ScriptCompiler::Source script_source(src, script_origin);

    auto module = v8::ScriptCompiler::CompileModule(iso, &script_source);
    if (module.IsEmpty()) {
        capture_exception(ctx, tc, ctx->parent->error_buf,
                          KLYRON_V8_ERROR_BUF_SIZE);
        return nullptr;
    }

    return new klyron_v8_module(iso, module.ToLocalChecked(), ctx->parent);
}

klyron_v8_result_t klyron_v8_module_instantiate(klyron_v8_context_t* ctx,
                                                 klyron_v8_module_t* module) {
    klyron_v8_result_t result = {false, {0}};
    if (!ctx || !module) return result;

    auto iso = get_iso(ctx);
    v8::Locker locker(iso);
    v8::Isolate::Scope iso_scope(iso);
    v8::HandleScope scope(iso);
    auto context = get_ctx(ctx);
    v8::Context::Scope ctx_scope(context);

    v8::TryCatch tc(iso);

    auto mod = module->module->Get(iso);
    auto status = mod->InstantiateModule(context, nullptr);

    if (status.IsNothing() || !status.FromJust()) {
        capture_exception(ctx, tc, result.error, KLYRON_V8_ERROR_BUF_SIZE);
        return result;
    }

    set_bool_result(&result, true);
    return result;
}

klyron_v8_string_result_t klyron_v8_module_evaluate(klyron_v8_context_t* ctx,
                                                     klyron_v8_module_t* module) {
    klyron_v8_string_result_t result = {false, nullptr, 0, {0}};
    if (!ctx || !module) return result;

    auto iso = get_iso(ctx);
    v8::Locker locker(iso);
    v8::Isolate::Scope iso_scope(iso);
    v8::HandleScope scope(iso);
    auto context = get_ctx(ctx);
    v8::Context::Scope ctx_scope(context);

    v8::TryCatch tc(iso);

    auto mod = module->module->Get(iso);
    auto eval_result = mod->Evaluate(context);

    if (eval_result.IsEmpty()) {
        capture_exception(ctx, tc, result.error, KLYRON_V8_ERROR_BUF_SIZE);
        return result;
    }

    auto val = eval_result.ToLocalChecked();
    if (val->IsPromise()) {
        auto promise = val.As<v8::Promise>();
        iso->PerformMicrotaskCheckpoint();
        if (promise->State() == v8::Promise::kRejected) {
            auto exc = promise->Result();
            v8::String::Utf8Value msg(iso, exc);
            std::strncpy(result.error, *msg ? *msg : "promise rejected",
                         KLYRON_V8_ERROR_BUF_SIZE - 1);
            return result;
        }
        val = promise->Result();
    }

    v8::String::Utf8Value utf8(iso, val);
    if (*utf8) {
        set_result(&result, std::string(*utf8, utf8.length()));
    } else {
        set_result(&result, "undefined");
    }
    return result;
}

int klyron_v8_module_get_identity(klyron_v8_context_t* ctx,
                                  klyron_v8_module_t* module) {
    if (!ctx || !module) return -1;
    auto iso = get_iso(ctx);
    v8::HandleScope scope(iso);
    auto mod = module->module->Get(iso);
    return static_cast<int>(mod->GetIdentityHash());
}

void klyron_v8_module_dispose(klyron_v8_module_t* module) {
    delete module;
}
