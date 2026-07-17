#include "klyron_v8.h"
#include "cpp/impl/internal.h"

klyron_v8_promise_t* klyron_v8_promise_new(klyron_v8_context_t* ctx) {
    if (!ctx) return nullptr;
    auto iso = get_iso(ctx);
    v8::Locker locker(iso);
    v8::Isolate::Scope iso_scope(iso);
    v8::HandleScope scope(iso);
    auto context = get_ctx(ctx);
    v8::Context::Scope ctx_scope(context);

    auto resolver = v8::Promise::Resolver::New(context);
    if (resolver.IsEmpty()) return nullptr;

    return new klyron_v8_promise(iso, resolver.ToLocalChecked(), ctx->parent);
}

klyron_v8_result_t klyron_v8_promise_resolve(klyron_v8_context_t* ctx,
                                             klyron_v8_promise_t* promise,
                                             klyron_v8_value_t* value) {
    klyron_v8_result_t result = {false, {0}};
    if (!ctx || !promise) return result;

    auto iso = get_iso(ctx);
    v8::Locker locker(iso);
    v8::Isolate::Scope iso_scope(iso);
    v8::HandleScope scope(iso);
    auto context = get_ctx(ctx);
    v8::Context::Scope ctx_scope(context);

    auto resolver = promise->resolver->Get(iso);
    auto val = value ? value->value->Get(iso) : v8::Undefined(iso).As<v8::Value>();
    auto status = resolver->Resolve(context, val);

    set_bool_result(&result, !status.IsNothing() && status.FromJust());
    return result;
}

klyron_v8_result_t klyron_v8_promise_reject(klyron_v8_context_t* ctx,
                                            klyron_v8_promise_t* promise,
                                            const char* reason) {
    klyron_v8_result_t result = {false, {0}};
    if (!ctx || !promise) return result;

    auto iso = get_iso(ctx);
    v8::Locker locker(iso);
    v8::Isolate::Scope iso_scope(iso);
    v8::HandleScope scope(iso);
    auto context = get_ctx(ctx);
    v8::Context::Scope ctx_scope(context);

    auto resolver = promise->resolver->Get(iso);
    auto msg =
        v8::String::NewFromUtf8(iso, reason, v8::NewStringType::kNormal)
            .ToLocalChecked();
    auto status = resolver->Reject(context, msg);

    set_bool_result(&result, !status.IsNothing() && status.FromJust());
    return result;
}

klyron_v8_value_t* klyron_v8_promise_get_native(klyron_v8_promise_t* promise) {
    (void)promise;
    return nullptr;
}

bool klyron_v8_promise_has_handler(klyron_v8_context_t* ctx,
                                   klyron_v8_promise_t* promise) {
    if (!ctx || !promise) return false;
    auto iso = get_iso(ctx);
    v8::Locker locker(iso);
    v8::HandleScope scope(iso);
    auto resolver = promise->resolver->Get(iso);
    return resolver->GetPromise()->HasHandler();
}

klyron_v8_result_t klyron_v8_promise_mark_as_handled(
    klyron_v8_context_t* ctx, klyron_v8_promise_t* promise) {
    klyron_v8_result_t result = {false, {0}};
    if (!ctx || !promise) return result;
    auto iso = get_iso(ctx);
    v8::Locker locker(iso);
    v8::HandleScope scope(iso);
    auto resolver = promise->resolver->Get(iso);
    resolver->GetPromise()->MarkAsHandled();
    set_bool_result(&result, true);
    return result;
}

klyron_v8_promise_state_t klyron_v8_promise_get_state(
    klyron_v8_context_t* ctx, klyron_v8_promise_t* promise) {
    if (!ctx || !promise) return KLYRON_V8_PROMISE_PENDING;
    auto iso = get_iso(ctx);
    v8::Locker locker(iso);
    v8::HandleScope scope(iso);
    auto resolver = promise->resolver->Get(iso);
    auto state = resolver->GetPromise()->State();
    switch (state) {
        case v8::Promise::kFulfilled: return KLYRON_V8_PROMISE_FULFILLED;
        case v8::Promise::kRejected:  return KLYRON_V8_PROMISE_REJECTED;
        default:                      return KLYRON_V8_PROMISE_PENDING;
    }
}


