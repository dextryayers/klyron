#include "klyron_v8.h"
#include "cpp/impl/internal.h"

#include <string>

klyron_v8_value_t* klyron_v8_get_exception(klyron_v8_context_t* ctx) {
    (void)ctx;
    return nullptr;
}

const char* klyron_v8_get_exception_message(klyron_v8_context_t* ctx) {
    if (!ctx) return "";
    auto iso = get_iso(ctx);
    v8::Locker locker(iso);
    v8::HandleScope scope(iso);

    auto context = get_ctx(ctx);
    v8::Context::Scope ctx_scope(context);

    auto exc = v8::String::NewFromUtf8(iso, ctx->parent->error_buf,
                                       v8::NewStringType::kNormal)
                   .ToLocalChecked();
    static std::string msg;
    v8::String::Utf8Value utf8(iso, exc);
    msg = *utf8 ? *utf8 : "";
    return msg.c_str();
}

klyron_v8_string_result_t klyron_v8_get_stack_trace(klyron_v8_context_t* ctx) {
    klyron_v8_string_result_t result = {false, nullptr, 0, {0}};
    if (!ctx) return result;

    auto iso = get_iso(ctx);
    v8::Locker locker(iso);
    v8::Isolate::Scope iso_scope(iso);
    v8::HandleScope scope(iso);
    auto context = get_ctx(ctx);
    v8::Context::Scope ctx_scope(context);

    auto stack = v8::StackTrace::CurrentStackTrace(iso, 50);
    if (stack.IsEmpty()) {
        set_error(&result, "no stack trace available");
        return result;
    }

    std::string trace;
    for (int i = 0; i < stack->GetFrameCount(); ++i) {
        auto frame = stack->GetFrame(iso, i);
        v8::String::Utf8Value func_name(iso, frame->GetFunctionName());
        v8::String::Utf8Value script_name(iso,
                                          frame->GetScriptNameOrSourceURL());
        int line = frame->GetLineNumber();
        int col = frame->GetColumn();

        trace += "  at ";
        trace += *func_name ? *func_name : "<anonymous>";
        trace += " (";
        trace += *script_name ? *script_name : "<unknown>";
        trace += ":" + std::to_string(line) + ":" + std::to_string(col) + ")\n";
    }

    set_result(&result, trace);
    return result;
}
