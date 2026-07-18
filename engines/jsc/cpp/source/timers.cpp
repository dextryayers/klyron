#include "klyron_jsc.h"
#include "cpp/impl/internal.h"
#include <cstring>
#include <cstdlib>
#include <map>
#include <functional>
#include <atomic>
#include <chrono>

struct TimerEntry {
    unsigned int id;
    bool recurring;
    bool active;
    double delay_ms;
    std::chrono::steady_clock::time_point last_fired;
};

/* Per-engine timer state */
struct TimerState {
    std::map<unsigned int, TimerEntry> timers;
    std::atomic<unsigned int> next_id{1};
};

static std::map<klyron_jsc_engine_t*, TimerState> g_engine_timers;
static std::mutex g_timers_mutex;

static TimerState& get_timer_state(klyron_jsc_engine_t* engine) {
    std::lock_guard<std::mutex> lock(g_timers_mutex);
    return g_engine_timers[engine];
}

static void execute_timer_cb(klyron_jsc_engine_t* engine, unsigned int id) {
    if (!engine || !engine->ctx) return;
    TimerState& state = get_timer_state(engine);
    auto it = state.timers.find(id);
    if (it == state.timers.end() || !it->second.active) return;
    JSObjectRef global = JSContextGetGlobalObject(engine->ctx);
    char cb_name[64];
    std::snprintf(cb_name, sizeof(cb_name), "__klyron_timer_%u", id);
    JSStringRef prop = jsc_string_from_cstr(cb_name);
    JSValueRef exc = nullptr;
    JSValueRef cb_val = JSObjectGetProperty(engine->ctx, global, prop, &exc);
    JSStringRelease(prop);
    if (!exc && cb_val && JSValueIsObject(engine->ctx, cb_val) && JSObjectIsFunction(engine->ctx, (JSObjectRef)cb_val)) {
        JSObjectRef func = (JSObjectRef)cb_val;
        JSValueRef call_exc = nullptr;
        JSObjectCallAsFunction(engine->ctx, func, global, 0, nullptr, &call_exc);
        if (call_exc) jsc_capture_exception(engine, call_exc);
    }
    if (!it->second.recurring) {
        state.timers.erase(id);
        char prop_name[64];
        std::snprintf(prop_name, sizeof(prop_name), "__klyron_timer_%u", id);
        JSStringRef del_prop = jsc_string_from_cstr(prop_name);
        JSObjectDeleteProperty(engine->ctx, global, del_prop, nullptr);
        JSStringRelease(del_prop);
    }
}

klyron_jsc_value_t* klyron_jsc_set_timeout(klyron_jsc_engine_t* engine, klyron_jsc_value_t* callback, double ms) {
    if (!engine || !callback) return nullptr;
    TimerState& state = get_timer_state(engine);
    unsigned int id = state.next_id.fetch_add(1, std::memory_order_seq_cst);
    state.timers[id] = {id, false, true, ms, std::chrono::steady_clock::now()};
    JSObjectRef global = JSContextGetGlobalObject(engine->ctx);
    char prop_name[64];
    std::snprintf(prop_name, sizeof(prop_name), "__klyron_timer_%u", id);
    JSStringRef prop = jsc_string_from_cstr(prop_name);
    JSValueRef exc = nullptr;
    JSObjectSetProperty(engine->ctx, global, prop, callback->value, kJSPropertyAttributeDontEnum, &exc);
    JSStringRelease(prop);
    if (exc) {
        jsc_capture_exception(engine, exc);
        state.timers.erase(id);
        return nullptr;
    }
    auto v = new klyron_jsc_value_t(engine->ctx, JSValueMakeNumber(engine->ctx, (double)id));
    v->protect();
    return v;
}

klyron_jsc_value_t* klyron_jsc_set_interval(klyron_jsc_engine_t* engine, klyron_jsc_value_t* callback, double ms) {
    if (!engine || !callback) return nullptr;
    TimerState& state = get_timer_state(engine);
    unsigned int id = state.next_id.fetch_add(1, std::memory_order_seq_cst);
    state.timers[id] = {id, true, true, ms, std::chrono::steady_clock::now()};
    JSObjectRef global = JSContextGetGlobalObject(engine->ctx);
    char prop_name[64];
    std::snprintf(prop_name, sizeof(prop_name), "__klyron_timer_%u", id);
    JSStringRef prop = jsc_string_from_cstr(prop_name);
    JSValueRef exc = nullptr;
    JSObjectSetProperty(engine->ctx, global, prop, callback->value, kJSPropertyAttributeDontEnum, &exc);
    JSStringRelease(prop);
    if (exc) {
        jsc_capture_exception(engine, exc);
        state.timers.erase(id);
        return nullptr;
    }
    auto v = new klyron_jsc_value_t(engine->ctx, JSValueMakeNumber(engine->ctx, (double)id));
    v->protect();
    return v;
}

klyron_jsc_value_t* klyron_jsc_set_immediate(klyron_jsc_engine_t* engine, klyron_jsc_value_t* callback) {
    return klyron_jsc_set_timeout(engine, callback, 0);
}

klyron_jsc_result_t klyron_jsc_clear_timeout(klyron_jsc_engine_t* engine, double id) {
    klyron_jsc_result_t result = {false, {0}};
    if (!engine) return result;
    TimerState& state = get_timer_state(engine);
    unsigned int uid = (unsigned int)id;
    auto it = state.timers.find(uid);
    if (it != state.timers.end()) {
        it->second.active = false;
        state.timers.erase(it);
        JSObjectRef global = JSContextGetGlobalObject(engine->ctx);
        char prop_name[64];
        std::snprintf(prop_name, sizeof(prop_name), "__klyron_timer_%u", uid);
        JSStringRef prop = jsc_string_from_cstr(prop_name);
        JSObjectDeleteProperty(engine->ctx, global, prop, nullptr);
        JSStringRelease(prop);
    }
    result.success = true;
    return result;
}

klyron_jsc_result_t klyron_jsc_clear_interval(klyron_jsc_engine_t* engine, double id) {
    return klyron_jsc_clear_timeout(engine, id);
}

klyron_jsc_result_t klyron_jsc_clear_immediate(klyron_jsc_engine_t* engine, double id) {
    return klyron_jsc_clear_timeout(engine, id);
}

void klyron_jsc_timer_poll(klyron_jsc_engine_t* engine) {
    if (!engine) return;
    TimerState& state = get_timer_state(engine);
    auto now = std::chrono::steady_clock::now();
    std::vector<unsigned int> to_execute;
    for (const auto& pair : state.timers) {
        if (!pair.second.active) continue;
        auto elapsed = std::chrono::duration_cast<std::chrono::milliseconds>(
            now - pair.second.last_fired).count();
        if (elapsed >= (long long)pair.second.delay_ms) {
            to_execute.push_back(pair.first);
        }
    }
    for (unsigned int id : to_execute) {
        execute_timer_cb(engine, id);
    }
}
