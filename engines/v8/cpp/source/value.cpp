#include "klyron_v8.h"
#include "cpp/impl/internal.h"

/* Value creation ------------------------------------------------------- */

klyron_v8_value_t* klyron_v8_value_new_string(klyron_v8_context_t* ctx,
                                               const char* str) {
    if (!ctx) return nullptr;
    auto iso = get_iso(ctx);
    v8::Locker locker(iso);
    v8::Isolate::Scope iso_scope(iso);
    v8::HandleScope scope(iso);
    auto val =
        v8::String::NewFromUtf8(iso, str, v8::NewStringType::kNormal)
            .ToLocalChecked();
    return new klyron_v8_value(iso, val, ctx->parent);
}

klyron_v8_value_t* klyron_v8_value_new_number(klyron_v8_context_t* ctx,
                                               double num) {
    if (!ctx) return nullptr;
    auto iso = get_iso(ctx);
    v8::Locker locker(iso);
    v8::Isolate::Scope iso_scope(iso);
    v8::HandleScope scope(iso);
    return new klyron_v8_value(iso, v8::Number::New(iso, num), ctx->parent);
}

klyron_v8_value_t* klyron_v8_value_new_bool(klyron_v8_context_t* ctx,
                                             bool val) {
    if (!ctx) return nullptr;
    auto iso = get_iso(ctx);
    v8::Locker locker(iso);
    v8::Isolate::Scope iso_scope(iso);
    v8::HandleScope scope(iso);
    return new klyron_v8_value(iso, v8::Boolean::New(iso, val), ctx->parent);
}

klyron_v8_value_t* klyron_v8_value_new_null(klyron_v8_context_t* ctx) {
    if (!ctx) return nullptr;
    auto iso = get_iso(ctx);
    v8::Locker locker(iso);
    v8::Isolate::Scope iso_scope(iso);
    v8::HandleScope scope(iso);
    return new klyron_v8_value(iso, v8::Null(iso), ctx->parent);
}

klyron_v8_value_t* klyron_v8_value_new_undefined(klyron_v8_context_t* ctx) {
    if (!ctx) return nullptr;
    auto iso = get_iso(ctx);
    v8::Locker locker(iso);
    v8::Isolate::Scope iso_scope(iso);
    v8::HandleScope scope(iso);
    return new klyron_v8_value(iso, v8::Undefined(iso), ctx->parent);
}

klyron_v8_value_t* klyron_v8_value_new_object(klyron_v8_context_t* ctx) {
    if (!ctx) return nullptr;
    auto iso = get_iso(ctx);
    v8::Locker locker(iso);
    v8::Isolate::Scope iso_scope(iso);
    v8::HandleScope scope(iso);
    auto context = get_ctx(ctx);
    v8::Context::Scope ctx_scope(context);
    return new klyron_v8_value(iso, v8::Object::New(iso), ctx->parent);
}

klyron_v8_value_t* klyron_v8_value_new_array(klyron_v8_context_t* ctx) {
    if (!ctx) return nullptr;
    auto iso = get_iso(ctx);
    v8::Locker locker(iso);
    v8::Isolate::Scope iso_scope(iso);
    v8::HandleScope scope(iso);
    auto context = get_ctx(ctx);
    v8::Context::Scope ctx_scope(context);
    return new klyron_v8_value(iso, v8::Array::New(iso), ctx->parent);
}

/* Value inspection ----------------------------------------------------- */

klyron_v8_type_result_t klyron_v8_value_typeof(klyron_v8_context_t* ctx,
                                                klyron_v8_value_t* value) {
    klyron_v8_type_result_t result = {KLYRON_V8_UNDEFINED, false, {0}};
    if (!ctx || !value) return result;

    auto iso = get_iso(ctx);
    v8::Locker locker(iso);
    v8::Isolate::Scope iso_scope(iso);
    v8::HandleScope scope(iso);
    auto context = get_ctx(ctx);
    v8::Context::Scope ctx_scope(context);

    auto val = value->value->Get(iso);
    result.success = true;

    if (val->IsUndefined())        result.type = KLYRON_V8_UNDEFINED;
    else if (val->IsNull())        result.type = KLYRON_V8_NULL;
    else if (val->IsBoolean())     result.type = KLYRON_V8_BOOLEAN;
    else if (val->IsNumber())      result.type = KLYRON_V8_NUMBER;
    else if (val->IsString())      result.type = KLYRON_V8_STRING;
    else if (val->IsFunction())    result.type = KLYRON_V8_FUNCTION;
    else if (val->IsArray())       result.type = KLYRON_V8_ARRAY;
    else if (val->IsPromise())     result.type = KLYRON_V8_PROMISE;
    else if (val->IsNativeError()) result.type = KLYRON_V8_ERROR;
    else if (val->IsSymbol())      result.type = KLYRON_V8_SYMBOL;
    else if (val->IsBigInt())      result.type = KLYRON_V8_BIGINT;
    else if (val->IsProxy())       result.type = KLYRON_V8_PROXY;
    else                           result.type = KLYRON_V8_OBJECT;

    return result;
}

klyron_v8_string_result_t klyron_v8_value_to_string(klyron_v8_context_t* ctx,
                                                     klyron_v8_value_t* value) {
    klyron_v8_string_result_t result = {false, nullptr, 0, {0}};
    if (!ctx || !value) return result;

    auto iso = get_iso(ctx);
    v8::Locker locker(iso);
    v8::Isolate::Scope iso_scope(iso);
    v8::HandleScope scope(iso);
    auto context = get_ctx(ctx);
    v8::Context::Scope ctx_scope(context);

    auto val = value->value->Get(iso);

    if (val->IsUndefined()) {
        set_result(&result, "undefined");
        return result;
    }
    if (val->IsNull()) {
        set_result(&result, "null");
        return result;
    }

    auto str_val = val->ToString(context);
    if (str_val.IsEmpty()) {
        set_error(&result, "cannot convert to string");
        return result;
    }

    v8::String::Utf8Value utf8(iso, str_val.ToLocalChecked());
    if (*utf8) {
        set_result(&result, std::string(*utf8, utf8.length()));
    } else {
        set_error(&result, "empty string conversion");
    }
    return result;
}

double klyron_v8_value_to_number(klyron_v8_context_t* ctx,
                                 klyron_v8_value_t* value) {
    if (!ctx || !value) return 0.0;
    auto iso = get_iso(ctx);
    v8::Locker locker(iso);
    v8::Isolate::Scope iso_scope(iso);
    v8::HandleScope scope(iso);
    auto context = get_ctx(ctx);
    v8::Context::Scope ctx_scope(context);
    auto val = value->value->Get(iso);
    auto num = val->ToNumber(context);
    if (num.IsEmpty()) return 0.0;
    return num.ToLocalChecked()->Value();
}

bool klyron_v8_value_to_bool(klyron_v8_context_t* ctx,
                             klyron_v8_value_t* value) {
    if (!ctx || !value) return false;
    auto iso = get_iso(ctx);
    v8::Locker locker(iso);
    v8::Isolate::Scope iso_scope(iso);
    v8::HandleScope scope(iso);
    auto val = value->value->Get(iso);
    return val->BooleanValue(iso);
}

bool klyron_v8_value_is_array(klyron_v8_context_t* ctx,
                              klyron_v8_value_t* value) {
    if (!ctx || !value) return false;
    auto iso = get_iso(ctx);
    v8::Locker locker(iso);
    v8::Isolate::Scope iso_scope(iso);
    v8::HandleScope scope(iso);
    auto val = value->value->Get(iso);
    return val->IsArray();
}

void klyron_v8_value_dispose(klyron_v8_value_t* value) {
    delete value;
}
