#include "klyron_jsc.h"
#include "cpp/impl/internal.h"

/*
 * JSC C API does not have native module support (no JSModuleEvaluate, etc).
 * We implement modules via script evaluation in a wrapped scope.
 * For module semantics, we wrap the source in an IIFE-like module pattern
 * using `export` handling at the JS level.
 */

klyron_jsc_value_t* klyron_jsc_module_compile(klyron_jsc_engine_t* engine,
                                                const char* source,
                                                const char* origin) {
    (void)origin;
    if (!engine || !source) return nullptr;

    std::string wrapped =
        "(function(exports, module) {\n" +
        std::string(source) +
        "\nreturn module.exports;\n})";

    JSStringRef src = jsc_string_from_cstr(wrapped.c_str());
    if (!src) return nullptr;

    JSValueRef exc = nullptr;
    JSValueRef fn = JSEvaluateScript(engine->ctx, src, nullptr,
                                      jsc_string_from_cstr("<module>"), 1, &exc);
    JSStringRelease(src);

    if (exc || !fn) {
        jsc_capture_exception(engine, exc);
        return nullptr;
    }

    auto v = new klyron_jsc_value_t(engine->ctx, fn);
    v->protect();
    return v;
}

klyron_jsc_result_t klyron_jsc_module_instantiate(klyron_jsc_engine_t* engine,
                                                    klyron_jsc_value_t* module) {
    klyron_jsc_result_t result = {false, {0}};
    if (!engine || !module) return result;
    jsc_set_bool_result(&result, true);
    return result;
}

klyron_jsc_string_result_t klyron_jsc_module_evaluate(klyron_jsc_engine_t* engine,
                                                        klyron_jsc_value_t* module) {
    klyron_jsc_string_result_t result = {false, nullptr, 0, {0}};
    if (!engine || !module || !JSObjectIsFunction(engine->ctx, (JSObjectRef)module->value)) {
        jsc_set_error(engine, "module_evaluate: not a function");
        return result;
    }

    JSObjectRef exports = JSObjectMake(engine->ctx, nullptr, nullptr);
    JSObjectRef mod_obj = JSObjectMake(engine->ctx, nullptr, nullptr);

    JSStringRef exports_str = jsc_string_from_cstr("exports");
    JSStringRef module_str = jsc_string_from_cstr("module");

    JSValueRef exc = nullptr;
    JSObjectSetProperty(engine->ctx, mod_obj, exports_str, exports,
                        kJSPropertyAttributeNone, &exc);
    JSStringRelease(exports_str);

    if (exc) {
        jsc_capture_exception(engine, exc);
        return result;
    }

    JSValueRef args[] = { exports, mod_obj };
    JSValueRef val = JSObjectCallAsFunction(engine->ctx, (JSObjectRef)module->value,
                                             nullptr, 2, args, &exc);

    JSStringRelease(module_str);

    if (exc) {
        jsc_capture_exception(engine, exc);
        return result;
    }

    std::string str = jsc_val_to_std(engine->ctx, val ? val : JSValueMakeUndefined(engine->ctx), &exc);
    if (exc) {
        jsc_capture_exception(engine, exc);
        return result;
    }
    jsc_set_string_result(&result, str);
    return result;
}

void klyron_jsc_module_dispose(klyron_jsc_value_t* module) {
    delete module;
}
