#include "klyron_jsc.h"
#include "cpp/impl/internal.h"

/*
 * Native JSClassRef support.
 * JSC allows defining custom classes with callbacks via JSClassCreate.
 * We expose a simplified interface for creating functions and constructors.
 */

/* ─── Native function creation ───────────────────────────────── */

klyron_jsc_value_t* klyron_jsc_function_new(klyron_jsc_engine_t* engine,
                                              const char* name,
                                              void* callback,
                                              void* user_data) {
    if (!engine || !engine->ctx) return nullptr;

    /*
     * We create a JSObject that stores the user_data as private data.
     * The callback is stored in a static dispatch table.
     * When the function is called, we look up the callback and invoke it.
     *
     * For now, we create an anonymous function and set a property with the name.
     */
    JSStringRef fn_name = name ? jsc_string_from_cstr(name) : nullptr;
    JSObjectRef fn = JSObjectMakeFunctionWithCallback(engine->ctx, fn_name,
        [](JSContextRef ctx, JSObjectRef function, JSObjectRef thisObject,
           size_t argumentCount, const JSValueRef arguments[], JSValueRef* exception) -> JSValueRef {
            /* Retrieve user_data from the function object's private data */
            void* data = JSObjectGetPrivate(function);
            if (!data) return JSValueMakeUndefined(ctx);
            /* Call the stored C callback */
            auto cb = reinterpret_cast<JSValueRef(*)(JSContextRef, JSObjectRef, JSObjectRef, size_t, const JSValueRef*, JSValueRef*)>(data);
            return cb(ctx, function, thisObject, argumentCount, arguments, exception);
        });
    if (fn_name) JSStringRelease(fn_name);

    /* Store user_data as private data on the function object */
    if (user_data) {
        JSObjectSetPrivate(fn, user_data);
    }

    auto v = new klyron_jsc_value_t(engine->ctx, fn);
    v->protect();
    return v;
}

/* ─── Native constructor creation ────────────────────────────── */

klyron_jsc_value_t* klyron_jsc_constructor_new(klyron_jsc_engine_t* engine,
                                                 const char* name,
                                                 void* callback,
                                                 void* user_data) {
    if (!engine || !engine->ctx) return nullptr;

    JSClassDefinition classDef = kJSClassDefinitionEmpty;
    classDef.className = name ? name : "AnonymousClass";
    classDef.callAsConstructor = 
        [](JSContextRef ctx, JSObjectRef constructor, size_t argumentCount,
           const JSValueRef arguments[], JSValueRef* exception) -> JSObjectRef {
            void* data = JSObjectGetPrivate(constructor);
            if (!data) return JSObjectMake(ctx, nullptr, nullptr);
            auto cb = reinterpret_cast<JSObjectRef(*)(JSContextRef, JSObjectRef, size_t, const JSValueRef*, JSValueRef*)>(data);
            return cb(ctx, constructor, argumentCount, arguments, exception);
        };

    JSClassRef jsClass = JSClassCreate(&classDef);
    JSObjectRef ctor = JSObjectMakeConstructor(engine->ctx, jsClass, nullptr);
    JSClassRelease(jsClass);

    if (user_data) {
        JSObjectSetPrivate(ctor, user_data);
    }

    auto v = new klyron_jsc_value_t(engine->ctx, ctor);
    v->protect();
    return v;
}

/* ─── Native error subclass creation ─────────────────────────── */

klyron_jsc_value_t* jsc_make_native_error(klyron_jsc_engine_t* engine,
                                           const char* constructor_name,
                                           const char* message) {
    if (!engine || !engine->ctx) return nullptr;

    /* Get the error constructor from the global object */
    JSStringRef ctor_str = jsc_string_from_cstr(constructor_name);
    JSValueRef ctor = JSObjectGetProperty(engine->ctx,
        JSContextGetGlobalObject(engine->ctx), ctor_str, nullptr);
    JSStringRelease(ctor_str);

    if (!ctor || !JSValueIsObject(engine->ctx, ctor)) {
        /* Fallback to eval for JSC versions without native constructor access */
        std::string code = std::string("new ") + constructor_name + "('" +
                           (message ? message : "") + "')";
        JSStringRef src = jsc_string_from_cstr(code.c_str());
        if (!src) return nullptr;
        JSValueRef exc = nullptr;
        JSValueRef val = JSEvaluateScript(engine->ctx, src, nullptr,
                                           jsc_string_from_cstr("<error>"), 1, &exc);
        JSStringRelease(src);
        if (exc || !val) {
            jsc_capture_exception(engine, exc);
            return nullptr;
        }
        auto v = new klyron_jsc_value_t(engine->ctx, val);
        v->protect();
        return v;
    }

    /* Create the error using the constructor */
    JSStringRef msg_str = jsc_string_from_cstr(message ? message : "");
    JSValueRef msg_val = JSValueMakeString(engine->ctx, msg_str);
    JSStringRelease(msg_str);
    JSValueRef args[] = { msg_val };
    JSValueRef exc = nullptr;
    JSObjectRef err = JSObjectCallAsConstructor(engine->ctx, (JSObjectRef)ctor,
                                                 1, args, &exc);
    if (exc || !err) {
        jsc_capture_exception(engine, exc);
        return nullptr;
    }

    auto v = new klyron_jsc_value_t(engine->ctx, err);
    v->protect();
    return v;
}

klyron_jsc_value_t* klyron_jsc_value_new_error(klyron_jsc_engine_t* engine,
                                                 const char* message) {
    /* Use JSObjectMakeError directly for plain Error */
    if (!engine || !engine->ctx) return nullptr;

    JSStringRef msg_str = jsc_string_from_cstr(message ? message : "");
    JSValueRef msg_val = JSValueMakeString(engine->ctx, msg_str);
    JSStringRelease(msg_str);
    JSValueRef args[] = { msg_val };
    JSValueRef exc = nullptr;
    JSObjectRef err = JSObjectMakeError(engine->ctx, 1, args, &exc);
    if (exc || !err) {
        jsc_capture_exception(engine, exc);
        return nullptr;
    }

    auto v = new klyron_jsc_value_t(engine->ctx, err);
    v->protect();
    return v;
}

klyron_jsc_value_t* klyron_jsc_value_new_type_error(klyron_jsc_engine_t* engine,
                                                      const char* message) {
    return jsc_make_native_error(engine, "TypeError", message);
}

klyron_jsc_value_t* klyron_jsc_value_new_range_error(klyron_jsc_engine_t* engine,
                                                       const char* message) {
    return jsc_make_native_error(engine, "RangeError", message);
}

klyron_jsc_value_t* klyron_jsc_value_new_syntax_error(klyron_jsc_engine_t* engine,
                                                        const char* message) {
    return jsc_make_native_error(engine, "SyntaxError", message);
}

klyron_jsc_value_t* klyron_jsc_value_new_reference_error(klyron_jsc_engine_t* engine,
                                                           const char* message) {
    return jsc_make_native_error(engine, "ReferenceError", message);
}
