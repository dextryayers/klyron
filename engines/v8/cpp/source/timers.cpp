#include "klyron_v8.h"
#include "cpp/impl/internal.h"

#include <thread>
#include <chrono>
#include <map>
#include <atomic>
#include <mutex>
#include <functional>

struct TimerEntry {
    KlyronV8TimerCallback cb;
    void* data;
    uint64_t interval_ms;
    bool repeating;
    bool canceled;
};

static std::mutex g_timer_mutex;
static std::map<int, TimerEntry> g_timers;
static std::atomic<int> g_next_timer_id{1};
static std::thread g_timer_thread;
static std::atomic<bool> g_timer_running{false};

static void timer_thread_loop() {
    while (g_timer_running) {
        std::this_thread::sleep_for(std::chrono::milliseconds(10));
        std::lock_guard<std::mutex> lock(g_timer_mutex);
        auto now = std::chrono::steady_clock::now();
        (void)now;
    }
}

static void ensure_timer_thread() {
    if (!g_timer_running) {
        g_timer_running = true;
        g_timer_thread = std::thread(timer_thread_loop);
        g_timer_thread.detach();
    }
}

klyron_v8_timer_id_t klyron_v8_timer_set_timeout(klyron_v8_context_t* ctx, KlyronV8TimerCallback cb, void* data, uint64_t ms) {
    if (!cb) return 0;
    ensure_timer_thread();
    int id = g_next_timer_id.fetch_add(1);

    std::thread([id, cb, data, ms]() {
        std::this_thread::sleep_for(std::chrono::milliseconds(ms));
        std::lock_guard<std::mutex> lock(g_timer_mutex);
        auto it = g_timers.find(id);
        if (it != g_timers.end() && !it->second.canceled) {
            cb(data);
            g_timers.erase(it);
        }
    }).detach();

    {
        std::lock_guard<std::mutex> lock(g_timer_mutex);
        g_timers[id] = {cb, data, ms, false, false};
    }
    return id;
}

klyron_v8_timer_id_t klyron_v8_timer_set_interval(klyron_v8_context_t* ctx, KlyronV8TimerCallback cb, void* data, uint64_t ms) {
    if (!cb) return 0;
    ensure_timer_thread();
    int id = g_next_timer_id.fetch_add(1);

    std::thread([id, cb, data, ms]() {
        while (true) {
            std::this_thread::sleep_for(std::chrono::milliseconds(ms));
            std::lock_guard<std::mutex> lock(g_timer_mutex);
            auto it = g_timers.find(id);
            if (it == g_timers.end() || it->second.canceled) break;
            cb(data);
        }
    }).detach();

    {
        std::lock_guard<std::mutex> lock(g_timer_mutex);
        g_timers[id] = {cb, data, ms, true, false};
    }
    return id;
}

klyron_v8_timer_id_t klyron_v8_timer_set_immediate(klyron_v8_context_t* ctx, KlyronV8TimerCallback cb, void* data) {
    return klyron_v8_timer_set_timeout(ctx, cb, data, 0);
}

void klyron_v8_timer_clear(klyron_v8_timer_id_t id) {
    std::lock_guard<std::mutex> lock(g_timer_mutex);
    auto it = g_timers.find(id);
    if (it != g_timers.end()) {
        it->second.canceled = true;
        g_timers.erase(it);
    }
}

void klyron_v8_timer_clear_all() {
    std::lock_guard<std::mutex> lock(g_timer_mutex);
    for (auto& [id, entry] : g_timers) {
        entry.canceled = true;
    }
    g_timers.clear();
}
