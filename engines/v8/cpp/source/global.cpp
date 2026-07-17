#include "klyron_v8.h"
#include "cpp/impl/internal.h"

klyron_v8_result_t klyron_v8_set_global(klyron_v8_context_t* ctx,
                                        const char* name,
                                        klyron_v8_value_t* value) {
    klyron_v8_result_t result = {false, {0}};
    if (!ctx || !name || !value) {
        set_bool_result(&result, false);
        return result;
    }

    auto iso = get_iso(ctx);
    v8::Locker locker(iso);
    v8::Isolate::Scope iso_scope(iso);
    v8::HandleScope scope(iso);
    auto context = get_ctx(ctx);
    v8::Context::Scope ctx_scope(context);

    auto global = context->Global();
    auto key =
        v8::String::NewFromUtf8(iso, name, v8::NewStringType::kNormal)
            .ToLocalChecked();
    auto val = value->value->Get(iso);

    global->Set(context, key, val).FromJust();

    set_bool_result(&result, true);
    return result;
}

klyron_v8_value_t* klyron_v8_get_global(klyron_v8_context_t* ctx,
                                        const char* name) {
    if (!ctx || !name) return nullptr;

    auto iso = get_iso(ctx);
    v8::Locker locker(iso);
    v8::Isolate::Scope iso_scope(iso);
    v8::HandleScope scope(iso);
    auto context = get_ctx(ctx);
    v8::Context::Scope ctx_scope(context);

    auto global = context->Global();
    auto key =
        v8::String::NewFromUtf8(iso, name, v8::NewStringType::kNormal)
            .ToLocalChecked();
    auto val = global->Get(context, key);

    if (val.IsEmpty()) return nullptr;

    return new klyron_v8_value(iso, val.ToLocalChecked(), ctx->parent);
}
