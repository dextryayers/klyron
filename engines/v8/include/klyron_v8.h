#ifndef KLYRON_V8_H
#define KLYRON_V8_H

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Opaque handle types */
typedef struct klyron_v8_isolate klyron_v8_isolate_t;
typedef struct klyron_v8_context klyron_v8_context_t;
typedef struct klyron_v8_value klyron_v8_value_t;
typedef struct klyron_v8_script klyron_v8_script_t;
typedef struct klyron_v8_module klyron_v8_module_t;
typedef struct klyron_v8_promise klyron_v8_promise_t;
typedef struct klyron_v8_snapshot klyron_v8_snapshot_t;

/* Error buffer size */
#define KLYRON_V8_ERROR_BUF_SIZE 4096

/* Heap statistics */
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
} klyron_v8_heap_stats_t;

/* Engine configuration */
typedef struct {
    const char* icu_data_path;
    const char* snapshot_blob_path;
    size_t max_heap_size_mb;
    size_t initial_heap_size_mb;
    uint32_t array_buffer_allocator_pool_size;
    bool use_shared_memory;
    bool expose_gc;
    bool single_threaded;
} klyron_v8_config_t;

/* Memory pressure level */
typedef enum {
    KLYRON_V8_MEMORY_PRESSURE_NONE = 0,
    KLYRON_V8_MEMORY_PRESSURE_MODERATE,
    KLYRON_V8_MEMORY_PRESSURE_CRITICAL
} klyron_v8_memory_pressure_t;

/* Value type tag */
typedef enum {
    KLYRON_V8_UNDEFINED = 0,
    KLYRON_V8_NULL,
    KLYRON_V8_BOOLEAN,
    KLYRON_V8_NUMBER,
    KLYRON_V8_STRING,
    KLYRON_V8_OBJECT,
    KLYRON_V8_ARRAY,
    KLYRON_V8_FUNCTION,
    KLYRON_V8_PROMISE,
    KLYRON_V8_ERROR,
    KLYRON_V8_SYMBOL,
    KLYRON_V8_BIGINT,
    KLYRON_V8_PROXY,
    KLYRON_V8_TYPED_ARRAY
} klyron_v8_value_type_t;

/* Result type */
typedef struct {
    bool success;
    char error[KLYRON_V8_ERROR_BUF_SIZE];
} klyron_v8_result_t;

/* String result */
typedef struct {
    bool success;
    char* data;
    size_t length;
    char error[KLYRON_V8_ERROR_BUF_SIZE];
} klyron_v8_string_result_t;

/* Value type query */
typedef struct {
    klyron_v8_value_type_t type;
    bool success;
    char error[KLYRON_V8_ERROR_BUF_SIZE];
} klyron_v8_type_result_t;

/*
 * Platform lifecycle
 */
void klyron_v8_init(const klyron_v8_config_t* config);
void klyron_v8_shutdown(void);
bool klyron_v8_is_initialized(void);

/*
 * Isolate lifecycle
 */
klyron_v8_isolate_t* klyron_v8_isolate_new(void);
void klyron_v8_isolate_dispose(klyron_v8_isolate_t* isolate);
void klyron_v8_isolate_enter(klyron_v8_isolate_t* isolate);
void klyron_v8_isolate_exit(klyron_v8_isolate_t* isolate);

/*
 * Context lifecycle
 */
klyron_v8_context_t* klyron_v8_context_new(klyron_v8_isolate_t* isolate);
void klyron_v8_context_dispose(klyron_v8_context_t* context);
void klyron_v8_context_enter(klyron_v8_context_t* context);
void klyron_v8_context_exit(klyron_v8_context_t* context);

/*
 * Script compilation and execution
 */
klyron_v8_script_t* klyron_v8_compile(klyron_v8_context_t* ctx, const char* source, const char* filename);
klyron_v8_string_result_t klyron_v8_run(klyron_v8_context_t* ctx, klyron_v8_script_t* script);
klyron_v8_string_result_t klyron_v8_eval(klyron_v8_context_t* ctx, const char* source, const char* filename);
void klyron_v8_script_dispose(klyron_v8_script_t* script);

/*
 * JSON operations
 */
klyron_v8_string_result_t klyron_v8_json_stringify(klyron_v8_context_t* ctx, klyron_v8_value_t* value);
klyron_v8_value_t* klyron_v8_json_parse(klyron_v8_context_t* ctx, const char* json);

/*
 * Global object access
 */
klyron_v8_result_t klyron_v8_set_global(klyron_v8_context_t* ctx, const char* name, klyron_v8_value_t* value);
klyron_v8_value_t* klyron_v8_get_global(klyron_v8_context_t* ctx, const char* name);

/*
 * Value creation
 */
klyron_v8_value_t* klyron_v8_value_new_string(klyron_v8_context_t* ctx, const char* str);
klyron_v8_value_t* klyron_v8_value_new_number(klyron_v8_context_t* ctx, double num);
klyron_v8_value_t* klyron_v8_value_new_bool(klyron_v8_context_t* ctx, bool val);
klyron_v8_value_t* klyron_v8_value_new_null(klyron_v8_context_t* ctx);
klyron_v8_value_t* klyron_v8_value_new_undefined(klyron_v8_context_t* ctx);
klyron_v8_value_t* klyron_v8_value_new_object(klyron_v8_context_t* ctx);
klyron_v8_value_t* klyron_v8_value_new_array(klyron_v8_context_t* ctx);

/*
 * Value inspection
 */
klyron_v8_type_result_t klyron_v8_value_typeof(klyron_v8_context_t* ctx, klyron_v8_value_t* value);
klyron_v8_string_result_t klyron_v8_value_to_string(klyron_v8_context_t* ctx, klyron_v8_value_t* value);
double klyron_v8_value_to_number(klyron_v8_context_t* ctx, klyron_v8_value_t* value);
bool klyron_v8_value_to_bool(klyron_v8_context_t* ctx, klyron_v8_value_t* value);
bool klyron_v8_value_is_array(klyron_v8_context_t* ctx, klyron_v8_value_t* value);

/*
 * Value disposal
 */
void klyron_v8_value_dispose(klyron_v8_value_t* value);

/*
 * Promise support
 */
klyron_v8_promise_t* klyron_v8_promise_new(klyron_v8_context_t* ctx);
klyron_v8_result_t klyron_v8_promise_resolve(klyron_v8_context_t* ctx, klyron_v8_promise_t* promise, klyron_v8_value_t* value);
klyron_v8_result_t klyron_v8_promise_reject(klyron_v8_context_t* ctx, klyron_v8_promise_t* promise, const char* reason);
klyron_v8_value_t* klyron_v8_promise_get_native(klyron_v8_promise_t* promise);
bool klyron_v8_promise_has_handler(klyron_v8_context_t* ctx, klyron_v8_promise_t* promise);
klyron_v8_result_t klyron_v8_promise_mark_as_handled(klyron_v8_context_t* ctx, klyron_v8_promise_t* promise);

/*
 * Promise state
 */
typedef enum {
    KLYRON_V8_PROMISE_PENDING = 0,
    KLYRON_V8_PROMISE_FULFILLED,
    KLYRON_V8_PROMISE_REJECTED
} klyron_v8_promise_state_t;

klyron_v8_promise_state_t klyron_v8_promise_get_state(klyron_v8_context_t* ctx, klyron_v8_promise_t* promise);

/*
 * Microtasks
 */
void klyron_v8_microtasks_perform_check(klyron_v8_context_t* ctx);

/*
 * Module support
 */
klyron_v8_module_t* klyron_v8_module_compile(klyron_v8_context_t* ctx, const char* source, const char* origin);
klyron_v8_result_t klyron_v8_module_instantiate(klyron_v8_context_t* ctx, klyron_v8_module_t* module);
klyron_v8_string_result_t klyron_v8_module_evaluate(klyron_v8_context_t* ctx, klyron_v8_module_t* module);
int klyron_v8_module_get_identity(klyron_v8_context_t* ctx, klyron_v8_module_t* module);
void klyron_v8_module_dispose(klyron_v8_module_t* module);

/*
 * Heap and memory management
 */
klyron_v8_result_t klyron_v8_get_heap_stats(klyron_v8_isolate_t* isolate, klyron_v8_heap_stats_t* stats);
void klyron_v8_low_memory_notification(klyron_v8_isolate_t* isolate);
void klyron_v8_idle_notification(klyron_v8_isolate_t* isolate, double deadline_in_seconds);
void klyron_v8_set_memory_pressure(klyron_v8_isolate_t* isolate, klyron_v8_memory_pressure_t pressure);
void klyron_v8_request_gc(klyron_v8_isolate_t* isolate);
size_t klyron_v8_get_malloced_memory(klyron_v8_isolate_t* isolate);
size_t klyron_v8_adjust_external_memory(klyron_v8_isolate_t* isolate, int64_t change);

/*
 * Snapshots
 */
klyron_v8_snapshot_t* klyron_v8_snapshot_create(klyron_v8_context_t* ctx);
klyron_v8_snapshot_t* klyron_v8_snapshot_load(const char* blob, size_t length);
void klyron_v8_snapshot_dispose(klyron_v8_snapshot_t* snapshot);

/*
 * Snapshot warm-start context
 */
klyron_v8_context_t* klyron_v8_context_new_from_snapshot(klyron_v8_isolate_t* isolate, klyron_v8_snapshot_t* snapshot);

/*
 * Error handling
 */
klyron_v8_value_t* klyron_v8_get_exception(klyron_v8_context_t* ctx);
const char* klyron_v8_get_exception_message(klyron_v8_context_t* ctx);
klyron_v8_string_result_t klyron_v8_get_stack_trace(klyron_v8_context_t* ctx);

/*
 * Memory management
 */
void klyron_v8_set_string_result(klyron_v8_string_result_t* result, const char* str);

/*
 * WebAssembly support
 */
klyron_v8_value_t* klyron_v8_wasm_compile(klyron_v8_context_t* ctx, const unsigned char* wasm_bytes, size_t wasm_length);
klyron_v8_value_t* klyron_v8_wasm_instantiate(klyron_v8_context_t* ctx, const unsigned char* wasm_bytes, size_t wasm_length, klyron_v8_value_t* imports);

/*
 * Inspector support
 */
int  klyron_v8_inspector_new(klyron_v8_isolate_t* isolate);
void klyron_v8_inspector_dispose(int inspector_id);
int  klyron_v8_inspector_connect(int inspector_id, const char* url);
void klyron_v8_inspector_disconnect(int session_id);
int  klyron_v8_inspector_dispatch(int session_id, const char* message, char* out_response, size_t out_response_size);
bool klyron_v8_inspector_is_active(void);

/*
 * Native function callback type
 */
typedef void (*KlyronV8FunctionCallback)(
    klyron_v8_context_t* ctx,
    int argc,
    klyron_v8_value_t** argv,
    void* user_data,
    klyron_v8_value_t** result
);

/*
 * Native function and constructor creation
 */
klyron_v8_value_t* klyron_v8_function_new(
    klyron_v8_context_t* ctx,
    const char* name,
    KlyronV8FunctionCallback callback,
    void* user_data
);

/*
 * Object property access (any object, not just global)
 */
klyron_v8_result_t klyron_v8_object_set_property(
    klyron_v8_context_t* ctx,
    klyron_v8_value_t* object,
    const char* name,
    klyron_v8_value_t* value
);
klyron_v8_value_t* klyron_v8_object_get_property(
    klyron_v8_context_t* ctx,
    klyron_v8_value_t* object,
    const char* name
);

/*
 * Typed array support
 */
typedef enum {
    KLYRON_V8_TYPED_ARRAY_NONE = 0,
    KLYRON_V8_TYPED_ARRAY_INT8,
    KLYRON_V8_TYPED_ARRAY_UINT8,
    KLYRON_V8_TYPED_ARRAY_UINT8_CLAMPED,
    KLYRON_V8_TYPED_ARRAY_INT16,
    KLYRON_V8_TYPED_ARRAY_UINT16,
    KLYRON_V8_TYPED_ARRAY_INT32,
    KLYRON_V8_TYPED_ARRAY_UINT32,
    KLYRON_V8_TYPED_ARRAY_FLOAT16,
    KLYRON_V8_TYPED_ARRAY_FLOAT32,
    KLYRON_V8_TYPED_ARRAY_FLOAT64,
    KLYRON_V8_TYPED_ARRAY_BIGINT64,
    KLYRON_V8_TYPED_ARRAY_BIGUINT64,
} klyron_v8_typed_array_type_t;

klyron_v8_typed_array_type_t klyron_v8_get_typed_array_type(
    klyron_v8_context_t* ctx, klyron_v8_value_t* value);
klyron_v8_value_t* klyron_v8_typed_array_new(
    klyron_v8_context_t* ctx, const char* type, size_t length);
size_t klyron_v8_typed_array_get_length(
    klyron_v8_context_t* ctx, klyron_v8_value_t* value);
klyron_v8_value_t* klyron_v8_typed_array_get_buffer(
    klyron_v8_context_t* ctx, klyron_v8_value_t* value);
klyron_v8_value_t* klyron_v8_array_buffer_new(
    klyron_v8_context_t* ctx, const unsigned char* data, size_t length);

/*
 * Buffer operations (Node.js Buffer-like)
 */
klyron_v8_value_t* klyron_v8_buffer_new(klyron_v8_context_t* ctx, size_t size);
klyron_v8_value_t* klyron_v8_buffer_from_string(klyron_v8_context_t* ctx, const char* str, const char* encoding);
klyron_v8_value_t* klyron_v8_buffer_from_bytes(klyron_v8_context_t* ctx, const unsigned char* data, size_t length);
klyron_v8_string_result_t klyron_v8_buffer_to_string(klyron_v8_context_t* ctx, klyron_v8_value_t* buf, const char* encoding, size_t start, size_t end);
unsigned char* klyron_v8_buffer_get_data(klyron_v8_context_t* ctx, klyron_v8_value_t* buf);
size_t klyron_v8_buffer_get_length(klyron_v8_context_t* ctx, klyron_v8_value_t* buf);
klyron_v8_result_t klyron_v8_buffer_copy(klyron_v8_context_t* ctx, klyron_v8_value_t* dst, size_t dst_offset, klyron_v8_value_t* src, size_t src_offset, size_t count);
klyron_v8_value_t* klyron_v8_buffer_concat(klyron_v8_context_t* ctx, klyron_v8_value_t** bufs, size_t count);
klyron_v8_value_t* klyron_v8_buffer_slice(klyron_v8_context_t* ctx, klyron_v8_value_t* buf, size_t start, size_t end);
klyron_v8_result_t klyron_v8_buffer_write(klyron_v8_context_t* ctx, klyron_v8_value_t* buf, const unsigned char* data, size_t offset, size_t length);

/*
 * Console API
 */
klyron_v8_value_t* klyron_v8_console_new(klyron_v8_context_t* ctx);
void klyron_v8_console_log(klyron_v8_context_t* ctx, const char* msg);
void klyron_v8_console_warn(klyron_v8_context_t* ctx, const char* msg);
void klyron_v8_console_error(klyron_v8_context_t* ctx, const char* msg);
void klyron_v8_console_info(klyron_v8_context_t* ctx, const char* msg);
void klyron_v8_console_debug(klyron_v8_context_t* ctx, const char* msg);

/*
 * Crypto primitives
 */
klyron_v8_value_t* klyron_v8_crypto_random_bytes(klyron_v8_context_t* ctx, size_t size);
klyron_v8_string_result_t klyron_v8_crypto_random_uuid(klyron_v8_context_t* ctx);

/*
 * Encoding (TextEncoder/TextDecoder, base64, hex)
 */
klyron_v8_value_t* klyron_v8_encoding_text_encoder_new(klyron_v8_context_t* ctx);
klyron_v8_value_t* klyron_v8_encoding_text_decoder_new(klyron_v8_context_t* ctx, const char* label);
klyron_v8_value_t* klyron_v8_encoding_encode(klyron_v8_context_t* ctx, const char* input);
klyron_v8_string_result_t klyron_v8_encoding_decode(klyron_v8_context_t* ctx, const unsigned char* data, size_t length, const char* encoding);
klyron_v8_string_result_t klyron_v8_encoding_base64_encode(klyron_v8_context_t* ctx, const unsigned char* data, size_t length);
klyron_v8_value_t* klyron_v8_encoding_base64_decode(klyron_v8_context_t* ctx, const char* input);
klyron_v8_string_result_t klyron_v8_encoding_hex_encode(klyron_v8_context_t* ctx, const unsigned char* data, size_t length);
klyron_v8_value_t* klyron_v8_encoding_hex_decode(klyron_v8_context_t* ctx, const char* input);

/*
 * Timers
 */
typedef void (*KlyronV8TimerCallback)(void* user_data);
typedef int klyron_v8_timer_id_t;
klyron_v8_timer_id_t klyron_v8_timer_set_timeout(klyron_v8_context_t* ctx, KlyronV8TimerCallback cb, void* data, uint64_t ms);
klyron_v8_timer_id_t klyron_v8_timer_set_interval(klyron_v8_context_t* ctx, KlyronV8TimerCallback cb, void* data, uint64_t ms);
klyron_v8_timer_id_t klyron_v8_timer_set_immediate(klyron_v8_context_t* ctx, KlyronV8TimerCallback cb, void* data);
void klyron_v8_timer_clear(klyron_v8_timer_id_t id);
void klyron_v8_timer_clear_all(void);

/*
 * URL parsing and resolution
 */
typedef struct {
    char* href;
    char* protocol;
    char* hostname;
    char* port;
    char* pathname;
    char* search;
    char* hash;
    char* host;
    char* origin;
} klyron_v8_url_t;

klyron_v8_url_t* klyron_v8_url_parse(const char* url, const char* base);
void klyron_v8_url_dispose(klyron_v8_url_t* url);

/*
 * File system operations
 */
typedef struct {
    uint64_t dev;
    uint64_t ino;
    uint32_t mode;
    uint32_t uid;
    uint32_t gid;
    uint64_t size;
    uint64_t blksize;
    uint64_t blocks;
    uint64_t atime;
    uint64_t mtime;
    uint64_t ctime;
    int32_t type; /* 0=file, 1=dir, 2=symlink, 3=other */
} klyron_v8_stat_t;

klyron_v8_result_t klyron_v8_fs_read_file(klyron_v8_context_t* ctx, const char* path, klyron_v8_value_t** result);
klyron_v8_result_t klyron_v8_fs_write_file(klyron_v8_context_t* ctx, const char* path, const unsigned char* data, size_t length);
klyron_v8_result_t klyron_v8_fs_append_file(klyron_v8_context_t* ctx, const char* path, const unsigned char* data, size_t length);
klyron_v8_result_t klyron_v8_fs_stat(klyron_v8_context_t* ctx, const char* path, klyron_v8_stat_t* stat);
klyron_v8_result_t klyron_v8_fs_mkdir(klyron_v8_context_t* ctx, const char* path, int32_t mode);
klyron_v8_result_t klyron_v8_fs_rmdir(klyron_v8_context_t* ctx, const char* path);
klyron_v8_result_t klyron_v8_fs_unlink(klyron_v8_context_t* ctx, const char* path);
klyron_v8_result_t klyron_v8_fs_rename(klyron_v8_context_t* ctx, const char* old_path, const char* new_path);
klyron_v8_result_t klyron_v8_fs_exists(klyron_v8_context_t* ctx, const char* path, bool* exists);
klyron_v8_value_t* klyron_v8_fs_read_dir(klyron_v8_context_t* ctx, const char* path);

/*
 * Process information
 */
typedef struct {
    char* exec_path;
    char** argv;
    int argc;
    char* cwd;
    char* platform;
    uint64_t pid;
    char* title;
    uint64_t ppid;
} klyron_v8_process_info_t;

klyron_v8_process_info_t* klyron_v8_process_info(klyron_v8_context_t* ctx);
klyron_v8_result_t klyron_v8_process_exit(klyron_v8_context_t* ctx, int code);
klyron_v8_string_result_t klyron_v8_process_env_get(klyron_v8_context_t* ctx, const char* name);
klyron_v8_value_t* klyron_v8_process_env_all(klyron_v8_context_t* ctx);
void klyron_v8_process_info_dispose(klyron_v8_process_info_t* info);

/*
 * Stream implementation
 */
typedef struct klyron_v8_stream klyron_v8_stream_t;

typedef size_t (*KlyronV8StreamReadCallback)(klyron_v8_stream_t* stream, unsigned char* buf, size_t count, void* user_data);
typedef size_t (*KlyronV8StreamWriteCallback)(klyron_v8_stream_t* stream, const unsigned char* buf, size_t count, void* user_data);
typedef void (*KlyronV8StreamCloseCallback)(klyron_v8_stream_t* stream, void* user_data);

klyron_v8_stream_t* klyron_v8_stream_new_readable(klyron_v8_context_t* ctx, KlyronV8StreamReadCallback read_cb, void* user_data);
klyron_v8_stream_t* klyron_v8_stream_new_writable(klyron_v8_context_t* ctx, KlyronV8StreamWriteCallback write_cb, void* user_data);
klyron_v8_stream_t* klyron_v8_stream_new_transform(klyron_v8_context_t* ctx, KlyronV8StreamReadCallback read_cb, KlyronV8StreamWriteCallback write_cb, void* user_data);
klyron_v8_result_t klyron_v8_stream_write(klyron_v8_context_t* ctx, klyron_v8_stream_t* stream, const unsigned char* data, size_t length);
klyron_v8_result_t klyron_v8_stream_end(klyron_v8_context_t* ctx, klyron_v8_stream_t* stream, const unsigned char* data, size_t length);
klyron_v8_result_t klyron_v8_stream_destroy(klyron_v8_context_t* ctx, klyron_v8_stream_t* stream);
void klyron_v8_stream_set_close_callback(klyron_v8_stream_t* stream, KlyronV8StreamCloseCallback cb, void* user_data);

/*
 * Utility
 */
const char* klyron_v8_version(void);
int klyron_v8_major_version(void);
int klyron_v8_minor_version(void);
int klyron_v8_build_version(void);
int klyron_v8_patch_version(void);

void klyron_v8_free_string(char* s);
void klyron_v8_free_buffer(unsigned char* buf);

#ifdef __cplusplus
}
#endif

#endif /* KLYRON_V8_H */
