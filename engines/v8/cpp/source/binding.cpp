#include "klyron_v8.h"
#include "cpp/impl/internal.h"

/*
 * Native function callback trampoline.
 * v8::FunctionCallback expects a specific signature:
 *   void callback(const v8::FunctionCallbackInfo<v8::Value>& info)
 *
 * We adapt from our C callback (KlyronV8FunctionCallback) via a
 * v8::FunctionCallback that unpacks user_data and forwards the call.
 */
namespace {

struct FunctionBinding {
    KlyronV8FunctionCallback cb;
    void* user_data;
    klyron_v8_isolate_t* isolate;
};

void function_trampoline(const v8::FunctionCallbackInfo<v8::Value>& info) {
    auto binding = static_cast<FunctionBinding*>(
        info.Data().As<v8::External>()->Value());
    if (!binding || !binding->cb) return;

    auto iso = binding->isolate;
    auto ctx_obj = new klyron_v8_context(info.GetIsolate(), iso);
    klyron_v8_value_t** argv = nullptr;

    /* Convert V8 arguments to our klyron_v8_value_t array */
    int argc = info.Length();
    if (argc > 0) {
        argv = static_cast<klyron_v8_value_t**>(
            std::malloc(sizeof(klyron_v8_value_t*) * argc));
        for (int i = 0; i < argc; i++) {
            argv[i] = new klyron_v8_value(info.GetIsolate(),
                                           info[i], iso);
        }
    }

    klyron_v8_value_t* result = nullptr;
    binding->cb(ctx_obj, argc, argv, binding->user_data, &result);

    /* Set return value */
    if (result && result->value) {
        info.GetReturnValue().Set(result->value->Get(info.GetIsolate()));
    }

    /* Cleanup */
    if (argv) {
        for (int i = 0; i < argc; i++) {
            if (argv[i]) delete argv[i];
        }
        std::free(argv);
    }
    delete ctx_obj;
}

void constructor_trampoline(const v8::FunctionCallbackInfo<v8::Value>& info) {
    auto binding = static_cast<FunctionBinding*>(
        info.Data().As<v8::External>()->Value());
    if (!binding || !binding->cb) return;

    if (!info.IsConstructCall()) {
        info.GetIsolate()->ThrowError("constructor requires 'new'");
        return;
    }

    auto iso = binding->isolate;
    auto ctx_obj = new klyron_v8_context(info.GetIsolate(), iso);

    int argc = info.Length();
    klyron_v8_value_t** argv = nullptr;
    if (argc > 0) {
        argv = static_cast<klyron_v8_value_t**>(
            std::malloc(sizeof(klyron_v8_value_t*) * argc));
        for (int i = 0; i < argc; i++) {
            argv[i] = new klyron_v8_value(info.GetIsolate(),
                                           info[i], iso);
        }
    }

    klyron_v8_value_t* result = nullptr;
    binding->cb(ctx_obj, argc, argv, binding->user_data, &result);

    if (result && result->value) {
        info.GetReturnValue().Set(result->value->Get(info.GetIsolate()));
    }

    if (argv) {
        for (int i = 0; i < argc; i++) {
            if (argv[i]) delete argv[i];
        }
        std::free(argv);
    }
    delete ctx_obj;
}

} /* anonymous namespace */

klyron_v8_value_t* klyron_v8_function_new(klyron_v8_context_t* ctx,
                                           const char* name,
                                           KlyronV8FunctionCallback callback,
                                           void* user_data) {
    if (!ctx || !callback) return nullptr;
    auto iso = get_iso(ctx);
    if (!iso) return nullptr;

    v8::Locker locker(iso);
    v8::Isolate::Scope iso_scope(iso);
    v8::HandleScope scope(iso);

    auto binding = new FunctionBinding{callback, user_data, ctx->parent};
    auto ext = v8::External::New(iso, binding);
    auto fn_name = name
        ? v8::String::NewFromUtf8(iso, name, v8::NewStringType::kNormal).ToLocalChecked()
        : v8::String::NewFromUtf8(iso, "", v8::NewStringType::kNormal).ToLocalChecked();

    auto tmpl = v8::FunctionTemplate::New(iso, function_trampoline, ext);
    tmpl->SetClassName(fn_name);
    auto fn = tmpl->GetFunction(get_ctx(ctx)).ToLocalChecked();

    return new klyron_v8_value(iso, fn, ctx->parent);
}

/* ─── Object property access ──────────────────────────────────── */

klyron_v8_result_t klyron_v8_object_set_property(klyron_v8_context_t* ctx,
                                                  klyron_v8_value_t* object,
                                                  const char* name,
                                                  klyron_v8_value_t* value) {
    klyron_v8_result_t result = {false, {0}};
    if (!ctx || !object || !name || !value) return result;

    auto iso = get_iso(ctx);
    v8::Locker locker(iso);
    v8::Isolate::Scope iso_scope(iso);
    v8::HandleScope scope(iso);
    auto context = get_ctx(ctx);
    v8::Context::Scope ctx_scope(context);

    auto obj = object->value->Get(iso);
    if (!obj->IsObject()) return result;

    auto key = v8::String::NewFromUtf8(iso, name, v8::NewStringType::kNormal)
                   .ToLocalChecked();
    auto val = value->value->Get(iso);

    auto r = obj.As<v8::Object>()->Set(context, key, val);
    set_bool_result(&result, !r.IsNothing() && r.FromJust());
    return result;
}

klyron_v8_value_t* klyron_v8_object_get_property(klyron_v8_context_t* ctx,
                                                  klyron_v8_value_t* object,
                                                  const char* name) {
    if (!ctx || !object || !name) return nullptr;

    auto iso = get_iso(ctx);
    v8::Locker locker(iso);
    v8::Isolate::Scope iso_scope(iso);
    v8::HandleScope scope(iso);
    auto context = get_ctx(ctx);
    v8::Context::Scope ctx_scope(context);

    auto obj = object->value->Get(iso);
    if (!obj->IsObject()) return nullptr;

    auto key = v8::String::NewFromUtf8(iso, name, v8::NewStringType::kNormal)
                   .ToLocalChecked();
    auto val = obj.As<v8::Object>()->Get(context, key);
    if (val.IsEmpty()) return nullptr;

    return new klyron_v8_value(iso, val.ToLocalChecked(), ctx->parent);
}
