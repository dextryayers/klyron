#include "quickjs_wrapper.h"
#include "../vendor/quickjs/quickjs.h"
#include <string.h>
#include <stdlib.h>
#include <stdio.h>

struct quickjs_engine {
    JSRuntime *rt;
    JSContext *ctx;
    char last_error[4096];
};

static void set_error(quickjs_engine *engine, const char *msg) {
    if (engine && msg) {
        strncpy(engine->last_error, msg, sizeof(engine->last_error) - 1);
        engine->last_error[sizeof(engine->last_error) - 1] = '\0';
    }
}

static char* jsval_to_json(JSContext *ctx, JSValueConst val) {
    JSValue json = JS_JSONStringify(ctx, val, JS_UNDEFINED, JS_UNDEFINED);
    if (JS_IsException(json)) {
        JS_FreeValue(ctx, json);
        return NULL;
    }
    size_t len;
    const char *str = JS_ToCStringLen(ctx, &len, json);
    if (!str) {
        JS_FreeValue(ctx, json);
        return NULL;
    }
    char *result = (char*)malloc(len + 1);
    if (result) {
        memcpy(result, str, len + 1);
    }
    JS_FreeCString(ctx, str);
    JS_FreeValue(ctx, json);
    return result;
}

static int capture_exception(quickjs_engine *engine) {
    JSValue exc = JS_GetException(engine->ctx);
    char *msg = jsval_to_json(engine->ctx, exc);
    if (msg) {
        set_error(engine, msg);
        free(msg);
    } else {
        JSValue msg_val = JS_GetPropertyStr(engine->ctx, exc, "message");
        const char *msg_str = JS_ToCString(engine->ctx, msg_val);
        if (msg_str) {
            set_error(engine, msg_str);
            JS_FreeCString(engine->ctx, msg_str);
        } else {
            set_error(engine, "Unknown script error");
        }
        JS_FreeValue(engine->ctx, msg_val);
    }
    JS_FreeValue(engine->ctx, exc);
    return 0;
}

quickjs_engine* quickjs_init(void) {
    quickjs_engine *engine = (quickjs_engine*)calloc(1, sizeof(quickjs_engine));
    if (!engine) return NULL;
    engine->last_error[0] = '\0';
    engine->rt = JS_NewRuntime();
    if (!engine->rt) {
        free(engine);
        return NULL;
    }
    JS_SetMaxStackSize(engine->rt, 1024 * 1024);
    JS_SetMemoryLimit(engine->rt, 512 * 1024 * 1024);
    engine->ctx = JS_NewContext(engine->rt);
    if (!engine->ctx) {
        JS_FreeRuntime(engine->rt);
        free(engine);
        return NULL;
    }
    return engine;
}

void quickjs_destroy(quickjs_engine* engine) {
    if (!engine) return;
    if (engine->ctx) JS_FreeContext(engine->ctx);
    if (engine->rt) JS_FreeRuntime(engine->rt);
    free(engine);
}

char* quickjs_eval(quickjs_engine* engine, const char* code) {
    if (!engine || !code) return NULL;
    JSValue val = JS_Eval(engine->ctx, code, strlen(code), "<eval>", JS_EVAL_TYPE_GLOBAL);
    if (JS_IsException(val)) {
        capture_exception(engine);
        JS_FreeValue(engine->ctx, val);
        return NULL;
    }
    char *result = jsval_to_json(engine->ctx, val);
    JS_FreeValue(engine->ctx, val);
    if (!result) set_error(engine, "Failed to convert result to JSON");
    return result;
}

char* quickjs_execute_script(quickjs_engine* engine, const char* filename, const char* source) {
    if (!engine || !filename || !source) return NULL;
    JSValue val = JS_Eval(engine->ctx, source, strlen(source), filename, JS_EVAL_TYPE_GLOBAL);
    if (JS_IsException(val)) {
        capture_exception(engine);
        JS_FreeValue(engine->ctx, val);
        return NULL;
    }
    char *result = jsval_to_json(engine->ctx, val);
    JS_FreeValue(engine->ctx, val);
    if (!result) set_error(engine, "Failed to convert result to JSON");
    return result;
}

char* quickjs_execute_module(quickjs_engine* engine, const char* filename, const char* source) {
    if (!engine || !filename || !source) return NULL;
    JSValue val = JS_Eval(engine->ctx, source, strlen(source), filename, JS_EVAL_TYPE_MODULE);
    if (JS_IsException(val)) {
        capture_exception(engine);
        JS_FreeValue(engine->ctx, val);
        return NULL;
    }
    char *result = jsval_to_json(engine->ctx, val);
    JS_FreeValue(engine->ctx, val);
    if (!result) set_error(engine, "Failed to convert result to JSON");
    return result;
}

char* quickjs_get_global(quickjs_engine* engine, const char* key) {
    if (!engine || !key) return NULL;
    JSValue global = JS_GetGlobalObject(engine->ctx);
    JSValue val = JS_GetPropertyStr(engine->ctx, global, key);
    JS_FreeValue(engine->ctx, global);
    if (JS_IsException(val)) {
        capture_exception(engine);
        JS_FreeValue(engine->ctx, val);
        return NULL;
    }
    char *result = jsval_to_json(engine->ctx, val);
    JS_FreeValue(engine->ctx, val);
    if (!result) set_error(engine, "Failed to convert global to JSON");
    return result;
}

int quickjs_set_global(quickjs_engine* engine, const char* key, const char* value) {
    if (!engine || !key || !value) return -1;
    JSValue global = JS_GetGlobalObject(engine->ctx);
    JSValue val = JS_ParseJSON(engine->ctx, value, strlen(value), "<set_global>");
    if (JS_IsException(val)) {
        JS_FreeValue(engine->ctx, val);
        val = JS_NewString(engine->ctx, value);
    }
    int ret = JS_SetPropertyStr(engine->ctx, global, key, val);
    JS_FreeValue(engine->ctx, global);
    return ret ? 0 : -1;
}

static JSValue json_to_jsval(JSContext *ctx, const char* json) {
    return JS_ParseJSON(ctx, json, strlen(json), "<call_arg>");
}

char* quickjs_call_function(quickjs_engine* engine, const char* name, const char** args, int argc) {
    if (!engine || !name) return NULL;
    JSValue global = JS_GetGlobalObject(engine->ctx);
    JSValue func = JS_GetPropertyStr(engine->ctx, global, name);
    JS_FreeValue(engine->ctx, global);
    if (JS_IsException(func)) {
        capture_exception(engine);
        JS_FreeValue(engine->ctx, func);
        return NULL;
    }
    if (!JS_IsFunction(engine->ctx, func)) {
        JS_FreeValue(engine->ctx, func);
        char buf[256];
        snprintf(buf, sizeof(buf), "Function '%s' not found", name);
        set_error(engine, buf);
        return NULL;
    }
    JSValue *argv = NULL;
    if (argc > 0) {
        argv = (JSValue*)malloc(sizeof(JSValue) * argc);
        for (int i = 0; i < argc; i++) {
            argv[i] = json_to_jsval(engine->ctx, args[i]);
        }
    }
    JSValue result = JS_Call(engine->ctx, func, JS_UNDEFINED, argc, argv);
    for (int i = 0; i < argc; i++) {
        JS_FreeValue(engine->ctx, argv[i]);
    }
    free(argv);
    JS_FreeValue(engine->ctx, func);
    if (JS_IsException(result)) {
        capture_exception(engine);
        JS_FreeValue(engine->ctx, result);
        return NULL;
    }
    char *json = jsval_to_json(engine->ctx, result);
    JS_FreeValue(engine->ctx, result);
    if (!json) set_error(engine, "Failed to convert call result to JSON");
    return json;
}

unsigned char* quickjs_create_snapshot(quickjs_engine* engine, size_t* out_len) {
    if (!engine || !out_len) return NULL;
    JSValue global = JS_GetGlobalObject(engine->ctx);
    size_t len;
    uint8_t *data = JS_WriteObject(engine->ctx, &len, global, JS_WRITE_OBJ_REFERENCE);
    JS_FreeValue(engine->ctx, global);
    if (data) {
        unsigned char *buf = (unsigned char*)malloc(len);
        if (buf) {
            memcpy(buf, data, len);
            *out_len = len;
        }
        js_free(engine->ctx, data);
        return buf;
    }
    set_error(engine, "Failed to create snapshot");
    *out_len = 0;
    return NULL;
}

int quickjs_load_snapshot(quickjs_engine* engine, const unsigned char* data, size_t len) {
    if (!engine || !data || len == 0) return -1;
    JSValue obj = JS_ReadObject(engine->ctx, data, len, 0);
    if (JS_IsException(obj)) {
        capture_exception(engine);
        JS_FreeValue(engine->ctx, obj);
        return -1;
    }
    JS_FreeValue(engine->ctx, obj);
    return 0;
}

const char* quickjs_last_error(quickjs_engine* engine) {
    if (!engine) return "Null engine";
    return engine->last_error;
}

void quickjs_free_string(char* s) {
    free(s);
}

void quickjs_free_buffer(unsigned char* buf) {
    free(buf);
}
