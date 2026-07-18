#include "klyron_v8.h"
#include "cpp/impl/internal.h"

#include <cstdlib>

class TrackingAllocator : public v8::ArrayBuffer::Allocator {
public:
    TrackingAllocator() = default;

    void* Allocate(size_t length) override {
        void* data = std::malloc(length);
        if (data) {
            g_array_buffer_total_allocated.fetch_add(
                length, std::memory_order_relaxed);
        }
        return data;
    }

    void* AllocateUninitialized(size_t length) override {
        void* data = std::malloc(length);
        if (data) {
            g_array_buffer_total_allocated.fetch_add(
                length, std::memory_order_relaxed);
        }
        return data;
    }

    void Free(void* data, size_t length) override {
        if (data) {
            g_array_buffer_total_allocated.fetch_sub(
                length, std::memory_order_relaxed);
            std::free(data);
        }
    }
};

static v8::ArrayBuffer::Allocator* s_allocator = nullptr;

v8::ArrayBuffer::Allocator* create_global_array_buffer_allocator(
    const klyron_v8_config_t* config) {
    if (s_allocator) return s_allocator;
    if (config && config->array_buffer_allocator_pool_size > 0) {
        s_allocator = new TrackingAllocator();
    } else {
        s_allocator = v8::ArrayBuffer::Allocator::NewDefaultAllocator();
    }
    return s_allocator;
}

void destroy_global_array_buffer_allocator(void) {
    delete s_allocator;
    s_allocator = nullptr;
}

v8::ArrayBuffer::Allocator* get_global_array_buffer_allocator(void) {
    return s_allocator;
}
