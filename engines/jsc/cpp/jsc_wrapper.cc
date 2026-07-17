#include "jsc_wrapper.h"
#include <JavaScriptCore/JavaScript.h>
#include <cstring>
#include <cstdlib>
#include <string>

struct jsc_engine {
    JSGlobalContextRef ctx;
    std::string last_error;

    jsc_engine() : ctx(nullptr) {}
    ~jsc_engine() {
        if (ctx) {
            JSGlobalContextRelease(ctx);
            ctx = nullptr;
        }
    }
};

static void set_error(jsc_engine* engine, const std::string& msg) {
    engine->last_error = msg;
}

static std::string jsstring_to_string(JSStringRef str) {
    size_t max_size = JSStringGetMaximumUTF8CStringSize(str);
    if (max_size == 0) return std::string();
    std::string result(max_size - 1, '\0');
    size_t written = JSStringGetUTF8CString(str, &result[0], max_size);
    result.resize(written);
    return result;
}

static std::string jsvalue_to_string(JSContextRef ctx, JSValueRef val, JSValueRef* exc) {
    JSStringRef str = JSValueToStringCopy(ctx, val, exc);
    if (!str) return std::string();
    std::string result = jsstring_to_string(str);
    JSStringRelease(str);
    return result;
}

static std::string get_exception(JSContextRef ctx, JSValueRef exc) {
    if (!exc) return "Unknown error";
    return jsvalue_to_string(ctx, exc, nullptr);
}

static JSStringRef cstr_to_jsstring(const char* s) {
    return JSStringCreateWithUTF8CString(s);
}

static char* alloc_c_string(const std::string& s) {
    char* buf = (char*)malloc(s.size() + 1);
    if (buf) {
        std::memcpy(buf, s.c_str(), s.size() + 1);
    }
    return buf;
}

jsc_engine* jsc_init(void) {
    jsc_engine* engine = new (std::nothrow) jsc_engine();
    if (!engine) return nullptr;

    engine->ctx = JSGlobalContextCreateInGroup(nullptr, nullptr);
    if (!engine->ctx) {
        delete engine;
        return nullptr;
    }

    return engine;
}

void jsc_destroy(jsc_engine* engine) {
    delete engine;
}

char* jsc_eval(jsc_engine* engine, const char* code) {
    if (!engine || !engine->ctx) return nullptr;

    JSValueRef exc = nullptr;
    JSStringRef script = cstr_to_jsstring(code);
    JSStringRef source_url = cstr_to_jsstring("<eval>");

    JSValueRef result = JSEvaluateScript(engine->ctx, script, nullptr, source_url, 1, &exc);

    JSStringRelease(source_url);
    JSStringRelease(script);

    if (exc) {
        set_error(engine, get_exception(engine->ctx, exc));
        return nullptr;
    }

    std::string str = jsvalue_to_string(engine->ctx, result, &exc);
    if (exc) {
        set_error(engine, get_exception(engine->ctx, exc));
        return nullptr;
    }

    return alloc_c_string(str);
}

char* jsc_execute_script(jsc_engine* engine, const char* filename, const char* source) {
    if (!engine || !engine->ctx) return nullptr;

    JSValueRef exc = nullptr;
    JSStringRef script = cstr_to_jsstring(source);
    JSStringRef source_url = cstr_to_jsstring(filename);

    JSValueRef result = JSEvaluateScript(engine->ctx, script, nullptr, source_url, 1, &exc);

    JSStringRelease(source_url);
    JSStringRelease(script);

    if (exc) {
        set_error(engine, get_exception(engine->ctx, exc));
        return nullptr;
    }

    std::string str = jsvalue_to_string(engine->ctx, result, &exc);
    if (exc) {
        set_error(engine, get_exception(engine->ctx, exc));
        return nullptr;
    }

    return alloc_c_string(str);
}

char* jsc_execute_module(jsc_engine* engine, const char* filename, const char* source) {
    return jsc_execute_script(engine, filename, source);
}

char* jsc_get_global(jsc_engine* engine, const char* key) {
    if (!engine || !engine->ctx) return nullptr;

    JSValueRef exc = nullptr;
    JSObjectRef global = JSContextGetGlobalObject(engine->ctx);
    JSStringRef prop = cstr_to_jsstring(key);

    JSValueRef val = JSObjectGetProperty(engine->ctx, global, prop, &exc);
    JSStringRelease(prop);

    if (exc) {
        set_error(engine, get_exception(engine->ctx, exc));
        return nullptr;
    }

    std::string str = jsvalue_to_string(engine->ctx, val, &exc);
    if (exc) {
        set_error(engine, get_exception(engine->ctx, exc));
        return nullptr;
    }

    return alloc_c_string(str);
}

int jsc_set_global(jsc_engine* engine, const char* key, const char* value) {
    if (!engine || !engine->ctx) return -1;

    JSValueRef exc = nullptr;
    JSObjectRef global = JSContextGetGlobalObject(engine->ctx);
    JSStringRef prop = cstr_to_jsstring(key);
    JSStringRef val_str = cstr_to_jsstring(value);
    JSValueRef val = JSValueMakeString(engine->ctx, val_str);

    JSObjectSetProperty(engine->ctx, global, prop, val, kJSPropertyAttributeNone, &exc);

    JSStringRelease(val_str);
    JSStringRelease(prop);

    if (exc) {
        set_error(engine, get_exception(engine->ctx, exc));
        return -1;
    }

    return 0;
}

char* jsc_call_function(jsc_engine* engine, const char* name, const char** args, int argc) {
    if (!engine || !engine->ctx) return nullptr;

    JSValueRef exc = nullptr;
    JSObjectRef global = JSContextGetGlobalObject(engine->ctx);
    JSStringRef prop = cstr_to_jsstring(name);

    JSValueRef func_val = JSObjectGetProperty(engine->ctx, global, prop, &exc);
    JSStringRelease(prop);

    if (exc) {
        set_error(engine, get_exception(engine->ctx, exc));
        return nullptr;
    }

    if (!JSValueIsObject(engine->ctx, func_val)) {
        set_error(engine, "Function not found: " + std::string(name));
        return nullptr;
    }

    JSObjectRef func = (JSObjectRef)func_val;
    std::vector<JSValueRef> argv;
    for (int i = 0; i < argc; i++) {
        JSStringRef arg_str = cstr_to_jsstring(args[i]);
        argv.push_back(JSValueMakeString(engine->ctx, arg_str));
        JSStringRelease(arg_str);
    }

    JSValueRef result = JSObjectCallAsFunction(engine->ctx, func, global, argc, argv.data(), &exc);

    if (exc) {
        set_error(engine, get_exception(engine->ctx, exc));
        return nullptr;
    }

    std::string str = jsvalue_to_string(engine->ctx, result, &exc);
    if (exc) {
        set_error(engine, get_exception(engine->ctx, exc));
        return nullptr;
    }

    return alloc_c_string(str);
}

unsigned char* jsc_create_snapshot(jsc_engine* engine, size_t* out_len) {
    if (!engine || !out_len) return nullptr;
    set_error(engine, "JSC snapshot creation not implemented");
    *out_len = 0;
    return nullptr;
}

int jsc_load_snapshot(jsc_engine* engine, const unsigned char* data, size_t len) {
    if (!engine || !data || len == 0) return -1;
    set_error(engine, "JSC snapshot loading not implemented");
    return -1;
}

const char* jsc_last_error(jsc_engine* engine) {
    if (!engine) return "Null engine";
    return engine->last_error.c_str();
}

void jsc_free_string(char* s) {
    free(s);
}

void jsc_free_buffer(unsigned char* buf) {
    free(buf);
}
