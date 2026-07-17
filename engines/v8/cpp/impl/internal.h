#ifndef KLYRON_V8_INTERNAL_H
#define KLYRON_V8_INTERNAL_H

#include "klyron_v8.h"
#include "cpp/impl/types.h"

#include <atomic>
#include <memory>
#include <mutex>

#include <libplatform/libplatform.h>

/* Global engine state -------------------------------------------------- */
extern std::mutex g_mutex;
extern bool g_initialized;
extern std::unique_ptr<v8::Platform> g_platform;
extern klyron_v8_config_t g_config;

/* Array-Buffer-Allocator ----------------------------------------------- */
extern std::atomic<size_t> g_array_buffer_total_allocated;

v8::ArrayBuffer::Allocator* create_global_array_buffer_allocator(
    const klyron_v8_config_t* config);
void destroy_global_array_buffer_allocator(void);
v8::ArrayBuffer::Allocator* get_global_array_buffer_allocator(void);

/* Internal helpers ----------------------------------------------------- */
v8::Isolate* get_iso(klyron_v8_context_t* ctx);
v8::MaybeLocal<v8::Context> get_ctx_maybe(klyron_v8_context_t* ctx);
v8::Local<v8::Context> get_ctx(klyron_v8_context_t* ctx);

void set_error(klyron_v8_string_result_t* r, const char* msg);
void set_result(klyron_v8_string_result_t* r, const std::string& s);
void set_bool_result(klyron_v8_result_t* r, bool ok);
void capture_exception(klyron_v8_context_t* ctx, v8::TryCatch& tc,
                       char* buf, size_t buf_size);

/* Microtask helpers ---------------------------------------------------- */
void enqueue_internal_microtask(void (*cb)(void*), void* data);
size_t drain_internal_microtasks(void);

#endif
