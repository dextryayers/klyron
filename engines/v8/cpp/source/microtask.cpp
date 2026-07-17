#include "klyron_v8.h"
#include "cpp/impl/internal.h"

#include <mutex>
#include <queue>

namespace {

struct MicrotaskEntry {
    void (*cb)(void*);
    void* data;
};

static std::mutex g_microtask_mutex;
static std::queue<MicrotaskEntry> g_microtask_queue;

}  // anonymous namespace

void enqueue_internal_microtask(void (*cb)(void*), void* data) {
    std::lock_guard<std::mutex> lock(g_microtask_mutex);
    g_microtask_queue.push({cb, data});
}

size_t drain_internal_microtasks(void) {
    size_t count = 0;
    std::lock_guard<std::mutex> lock(g_microtask_mutex);
    while (!g_microtask_queue.empty()) {
        auto entry = g_microtask_queue.front();
        g_microtask_queue.pop();
        if (entry.cb) {
            entry.cb(entry.data);
        }
        ++count;
    }
    return count;
}

void klyron_v8_microtasks_perform_check(klyron_v8_context_t* ctx) {
    if (!ctx) return;
    auto iso = get_iso(ctx);
    v8::Locker locker(iso);
    v8::Isolate::Scope iso_scope(iso);
    iso->PerformMicrotaskCheckpoint();
}
