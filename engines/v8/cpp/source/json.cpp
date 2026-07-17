#include "klyron_v8.h"
#include "cpp/impl/internal.h"

klyron_v8_string_result_t klyron_v8_json_stringify(klyron_v8_context_t* ctx,
                                                    klyron_v8_value_t* value) {
    klyron_v8_string_result_t result = {false, nullptr, 0, {0}};
    if (!ctx || !value) {
        set_error(&result, "null context or value");
        return result;
    }

    auto iso = get_iso(ctx);
    v8::Locker locker(iso);
    v8::Isolate::Scope iso_scope(iso);
    v8::HandleScope scope(iso);
    auto context = get_ctx(ctx);
    v8::Context::Scope ctx_scope(context);

    auto local_val = value->value->Get(iso);
    auto json_str = v8::JSON::Stringify(context, local_val);

    if (json_str.IsEmpty()) {
        set_error(&result, "JSON stringify failed");
        return result;
    }

    v8::String::Utf8Value utf8(iso, json_str.ToLocalChecked());
    if (*utf8) {
        set_result(&result, std::string(*utf8, utf8.length()));
    }
    return result;
}

klyron_v8_value_t* klyron_v8_json_parse(klyron_v8_context_t* ctx,
                                        const char* json) {
    if (!ctx) return nullptr;

    auto iso = get_iso(ctx);
    v8::Locker locker(iso);
    v8::Isolate::Scope iso_scope(iso);
    v8::HandleScope scope(iso);
    auto context = get_ctx(ctx);
    v8::Context::Scope ctx_scope(context);

    auto json_str =
        v8::String::NewFromUtf8(iso, json, v8::NewStringType::kNormal)
            .ToLocalChecked();
    auto parsed = v8::JSON::Parse(context, json_str);

    if (parsed.IsEmpty()) return nullptr;

    return new klyron_v8_value(iso, parsed.ToLocalChecked(), ctx->parent);
}
