#ifndef KLYRON_JSC_H
#define KLYRON_JSC_H

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Opaque handle types */
typedef struct klyron_jsc_engine  klyron_jsc_engine_t;
typedef struct klyron_jsc_value   klyron_jsc_value_t;
typedef struct klyron_jsc_script  klyron_jsc_script_t;

/* Error buffer size */
#define KLYRON_JSC_ERROR_BUF_SIZE 4096

/* Heap statistics (limited in JSC C API — most fields stubbed) */
typedef struct {
    size_t total_heap_size;
    size_t total_heap_size_executable;
    size_t total_physical_size;
    size_t total_available_size;
    size_t used_heap_size;
    size_t heap_size_limit;
    size_t malloced_memory;
    size_t peak_malloced_memory;
    size_t number_of_native_contexts;
    size_t number_of_detached_contexts;
    size_t total_global_handles_size;
    size_t used_global_handles_size;
    size_t external_memory;
} klyron_jsc_heap_stats_t;

/* Value type tag */
typedef enum {
    KLYRON_JSC_UNDEFINED = 0,
    KLYRON_JSC_NULL,
    KLYRON_JSC_BOOLEAN,
    KLYRON_JSC_NUMBER,
    KLYRON_JSC_STRING,
    KLYRON_JSC_OBJECT,
    KLYRON_JSC_ARRAY,
    KLYRON_JSC_FUNCTION,
    KLYRON_JSC_ERROR,
    KLYRON_JSC_SYMBOL,
    KLYRON_JSC_TYPED_ARRAY
} klyron_jsc_value_type_t;

/* Result type */
typedef struct {
    bool success;
    char error[KLYRON_JSC_ERROR_BUF_SIZE];
} klyron_jsc_result_t;

/* String result */
typedef struct {
    bool success;
    char* data;
    size_t length;
    char error[KLYRON_JSC_ERROR_BUF_SIZE];
} klyron_jsc_string_result_t;

/* Value type query result */
typedef struct {
    klyron_jsc_value_type_t type;
    bool success;
    char error[KLYRON_JSC_ERROR_BUF_SIZE];
} klyron_jsc_type_result_t;

/*
 * Engine lifecycle
 */
klyron_jsc_engine_t* klyron_jsc_init(void);
void klyron_jsc_shutdown(klyron_jsc_engine_t* engine);

/*
 * Script compilation and execution
 */
klyron_jsc_script_t*  klyron_jsc_compile(klyron_jsc_engine_t* engine, const char* source, const char* filename);
klyron_jsc_string_result_t klyron_jsc_run(klyron_jsc_engine_t* engine, klyron_jsc_script_t* script);
klyron_jsc_string_result_t klyron_jsc_eval(klyron_jsc_engine_t* engine, const char* source, const char* filename);
void klyron_jsc_script_dispose(klyron_jsc_script_t* script);

/*
 * Call function (with JSValueRef arguments)
 */
klyron_jsc_value_t* klyron_jsc_call_function(
    klyron_jsc_engine_t* engine,
    klyron_jsc_value_t* func,
    klyron_jsc_value_t* this_obj,
    int argc,
    klyron_jsc_value_t** argv
);

/*
 * Object property access (arbitrary objects)
 */
klyron_jsc_result_t klyron_jsc_object_set_property(
    klyron_jsc_engine_t* engine,
    klyron_jsc_value_t* object,
    const char* name,
    klyron_jsc_value_t* value
);
klyron_jsc_value_t* klyron_jsc_object_get_property(
    klyron_jsc_engine_t* engine,
    klyron_jsc_value_t* object,
    const char* name
);

/*
 * JSON operations
 */
klyron_jsc_string_result_t klyron_jsc_json_stringify(klyron_jsc_engine_t* engine, klyron_jsc_value_t* value);
klyron_jsc_value_t*        klyron_jsc_json_parse(klyron_jsc_engine_t* engine, const char* json);

/*
 * Global object access
 */
klyron_jsc_result_t klyron_jsc_set_global(klyron_jsc_engine_t* engine, const char* name, klyron_jsc_value_t* value);
klyron_jsc_value_t* klyron_jsc_get_global(klyron_jsc_engine_t* engine, const char* name);

/*
 * Value creation
 */
klyron_jsc_value_t* klyron_jsc_value_new_string(klyron_jsc_engine_t* engine, const char* str);
klyron_jsc_value_t* klyron_jsc_value_new_number(klyron_jsc_engine_t* engine, double num);
klyron_jsc_value_t* klyron_jsc_value_new_bool(klyron_jsc_engine_t* engine, bool val);
klyron_jsc_value_t* klyron_jsc_value_new_null(klyron_jsc_engine_t* engine);
klyron_jsc_value_t* klyron_jsc_value_new_undefined(klyron_jsc_engine_t* engine);
klyron_jsc_value_t* klyron_jsc_value_new_object(klyron_jsc_engine_t* engine);
klyron_jsc_value_t* klyron_jsc_value_new_array(klyron_jsc_engine_t* engine);
klyron_jsc_value_t* klyron_jsc_value_new_symbol(klyron_jsc_engine_t* engine, const char* description);
klyron_jsc_value_t* klyron_jsc_value_new_error(klyron_jsc_engine_t* engine, const char* message);

/*
 * Value inspection
 */
klyron_jsc_type_result_t    klyron_jsc_value_typeof(klyron_jsc_engine_t* engine, klyron_jsc_value_t* value);
klyron_jsc_string_result_t  klyron_jsc_value_to_string(klyron_jsc_engine_t* engine, klyron_jsc_value_t* value);
double klyron_jsc_value_to_number(klyron_jsc_engine_t* engine, klyron_jsc_value_t* value);
bool   klyron_jsc_value_to_bool(klyron_jsc_engine_t* engine, klyron_jsc_value_t* value);
bool   klyron_jsc_value_is_array(klyron_jsc_engine_t* engine, klyron_jsc_value_t* value);
bool   klyron_jsc_value_is_function(klyron_jsc_engine_t* engine, klyron_jsc_value_t* value);
bool   klyron_jsc_value_is_object(klyron_jsc_engine_t* engine, klyron_jsc_value_t* value);
bool   klyron_jsc_value_is_error(klyron_jsc_engine_t* engine, klyron_jsc_value_t* value);
bool   klyron_jsc_value_is_symbol(klyron_jsc_engine_t* engine, klyron_jsc_value_t* value);
bool   klyron_jsc_value_is_promise(klyron_jsc_engine_t* engine, klyron_jsc_value_t* value);
bool   klyron_jsc_value_is_typed_array(klyron_jsc_engine_t* engine, klyron_jsc_value_t* value);
bool   klyron_jsc_value_is_null(klyron_jsc_engine_t* engine, klyron_jsc_value_t* value);
bool   klyron_jsc_value_is_undefined(klyron_jsc_engine_t* engine, klyron_jsc_value_t* value);
bool   klyron_jsc_value_is_boolean(klyron_jsc_engine_t* engine, klyron_jsc_value_t* value);
bool   klyron_jsc_value_is_number(klyron_jsc_engine_t* engine, klyron_jsc_value_t* value);
bool   klyron_jsc_value_is_string(klyron_jsc_engine_t* engine, klyron_jsc_value_t* value);

/*
 * Value disposal
 */
void klyron_jsc_value_dispose(klyron_jsc_value_t* value);

/*
 * Promise support (via JS-level workaround)
 */
klyron_jsc_value_t* klyron_jsc_promise_new(klyron_jsc_engine_t* engine);
klyron_jsc_result_t klyron_jsc_promise_resolve(klyron_jsc_engine_t* engine, klyron_jsc_value_t* promise, klyron_jsc_value_t* value);
klyron_jsc_result_t klyron_jsc_promise_reject(klyron_jsc_engine_t* engine, klyron_jsc_value_t* promise, const char* reason);

/*
 * Microtasks
 */
void klyron_jsc_microtasks_perform_check(klyron_jsc_engine_t* engine);

/*
 * Module support (via eval)
 */
klyron_jsc_value_t*       klyron_jsc_module_compile(klyron_jsc_engine_t* engine, const char* source, const char* origin);
klyron_jsc_result_t       klyron_jsc_module_instantiate(klyron_jsc_engine_t* engine, klyron_jsc_value_t* module);
klyron_jsc_string_result_t klyron_jsc_module_evaluate(klyron_jsc_engine_t* engine, klyron_jsc_value_t* module);
void klyron_jsc_module_dispose(klyron_jsc_value_t* module);

/*
 * Heap and memory management
 */
klyron_jsc_result_t klyron_jsc_get_heap_stats(klyron_jsc_engine_t* engine, klyron_jsc_heap_stats_t* stats);
void klyron_jsc_request_gc(klyron_jsc_engine_t* engine);
void klyron_jsc_low_memory_notification(klyron_jsc_engine_t* engine);

/*
 * ArrayBuffer / TypedArray support
 */
klyron_jsc_value_t* klyron_jsc_array_buffer_new(klyron_jsc_engine_t* engine, const unsigned char* data, size_t length);
unsigned char*      klyron_jsc_array_buffer_get_data(klyron_jsc_engine_t* engine, klyron_jsc_value_t* value, size_t* out_length);
klyron_jsc_value_t* klyron_jsc_typed_array_new(klyron_jsc_engine_t* engine, const char* type, size_t length);
size_t              klyron_jsc_typed_array_get_length(klyron_jsc_engine_t* engine, klyron_jsc_value_t* value);
klyron_jsc_value_t* klyron_jsc_typed_array_get_buffer(klyron_jsc_engine_t* engine, klyron_jsc_value_t* value);

/*
 * WebAssembly support (via eval, no native C API in JSC)
 */
klyron_jsc_value_t* klyron_jsc_wasm_compile(klyron_jsc_engine_t* engine, const unsigned char* wasm_bytes, size_t wasm_length);
klyron_jsc_value_t* klyron_jsc_wasm_instantiate(klyron_jsc_engine_t* engine, const unsigned char* wasm_bytes, size_t wasm_length, klyron_jsc_value_t* imports);

/*
 * Error handling
 */
const char* klyron_jsc_get_exception_message(klyron_jsc_engine_t* engine);
klyron_jsc_string_result_t klyron_jsc_get_stack_trace(klyron_jsc_engine_t* engine);

/*
 * Utility
 */
const char* klyron_jsc_version(void);
void klyron_jsc_free_string(char* s);
void klyron_jsc_free_buffer(unsigned char* buf);

#ifdef __cplusplus
}
#endif

#endif /* KLYRON_JSC_H */
