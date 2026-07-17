#include "klyron_jsc.h"
#include "cpp/impl/internal.h"

/*
 * JSC C API does not expose native Wasm compilation.
 * WebAssembly is available at the JS level via eval.
 * These stubs allow calling Wasm through JS evaluation.
 */

klyron_jsc_value_t* klyron_jsc_wasm_compile(klyron_jsc_engine_t* engine,
                                              const unsigned char* wasm_bytes,
                                              size_t wasm_length) {
    if (!engine || !engine->ctx || !wasm_bytes || wasm_length == 0) return nullptr;

    std::string code =
        "var bytes = new Uint8Array([";
    for (size_t i = 0; i < wasm_length; i++) {
        char buf[8];
        std::snprintf(buf, sizeof(buf), "%s%d", i > 0 ? "," : "", wasm_bytes[i]);
        code += buf;
    }
    code += "]); WebAssembly.compile(bytes);";

    JSStringRef src = jsc_string_from_cstr(code.c_str());
    if (!src) return nullptr;

    JSValueRef exc = nullptr;
    JSValueRef val = JSEvaluateScript(engine->ctx, src, nullptr,
                                       jsc_string_from_cstr("<wasm>"), 1, &exc);
    JSStringRelease(src);

    if (exc || !val) {
        jsc_capture_exception(engine, exc);
        return nullptr;
    }

    auto v = new klyron_jsc_value_t(engine->ctx, val);
    v->protect();
    return v;
}

klyron_jsc_value_t* klyron_jsc_wasm_instantiate(klyron_jsc_engine_t* engine,
                                                  const unsigned char* wasm_bytes,
                                                  size_t wasm_length,
                                                  klyron_jsc_value_t* imports) {
    if (!engine || !engine->ctx || !wasm_bytes || wasm_length == 0) return nullptr;

    std::string code =
        "var bytes = new Uint8Array([";
    for (size_t i = 0; i < wasm_length; i++) {
        char buf[8];
        std::snprintf(buf, sizeof(buf), "%s%d", i > 0 ? "," : "", wasm_bytes[i]);
        code += buf;
    }
    code += "]); WebAssembly.instantiate(bytes);";

    JSStringRef src = jsc_string_from_cstr(code.c_str());
    if (!src) return nullptr;

    JSValueRef exc = nullptr;
    JSValueRef val = JSEvaluateScript(engine->ctx, src, nullptr,
                                       jsc_string_from_cstr("<wasm>"), 1, &exc);
    JSStringRelease(src);

    if (exc || !val) {
        jsc_capture_exception(engine, exc);
        return nullptr;
    }

    auto v = new klyron_jsc_value_t(engine->ctx, val);
    v->protect();
    return v;
}
