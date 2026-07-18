#include "klyron_v8.h"
#include "cpp/impl/internal.h"

klyron_v8_isolate_t* klyron_v8_isolate_new(void) {
    v8::Isolate::CreateParams params;

    if (g_config.max_heap_size_mb > 0) {
        params.constraints.set_max_old_generation_size_in_bytes(
            static_cast<size_t>(g_config.max_heap_size_mb) * 1024 * 1024);
    }
    if (g_config.initial_heap_size_mb > 0) {
        params.constraints.set_max_old_generation_size_in_bytes(
            static_cast<size_t>(g_config.initial_heap_size_mb) * 1024 * 1024);
    }

    auto alloc = get_global_array_buffer_allocator();
    if (!alloc) {
        /* V8 13.6 requires a non-null array_buffer_allocator */
        alloc = v8::ArrayBuffer::Allocator::NewDefaultAllocator();
    }
    params.array_buffer_allocator = alloc;

    auto isolate = v8::Isolate::New(params);
    if (!isolate) return nullptr;

    return new klyron_v8_isolate(isolate, true);
}

void klyron_v8_isolate_dispose(klyron_v8_isolate_t* isolate) {
    if (!isolate) return;
    delete isolate;
}

void klyron_v8_isolate_enter(klyron_v8_isolate_t* isolate) {
    if (isolate && isolate->isolate) isolate->isolate->Enter();
}

void klyron_v8_isolate_exit(klyron_v8_isolate_t* isolate) {
    if (isolate && isolate->isolate) isolate->isolate->Exit();
}
