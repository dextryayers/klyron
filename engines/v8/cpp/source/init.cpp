#include "klyron_v8.h"
#include "cpp/impl/internal.h"

#include <cstring>

std::mutex g_mutex;
bool g_initialized = false;
std::unique_ptr<v8::Platform> g_platform;
klyron_v8_config_t g_config;

std::atomic<size_t> g_array_buffer_total_allocated{0};

static void init_config(const klyron_v8_config_t* config) {
    if (config) {
        g_config = *config;
    } else {
        std::memset(&g_config, 0, sizeof(g_config));
    }
}

static void init_platform(void) {
    if (g_config.icu_data_path) {
        v8::V8::InitializeICUDefaultLocation(g_config.icu_data_path);
    }
    if (g_config.expose_gc) {
        v8::V8::SetFlagsFromString("--expose_gc");
    }
    g_platform = v8::platform::NewDefaultPlatform(
        g_config.single_threaded ? 0 : 1,
        v8::platform::IdleTaskSupport::kDisabled,
        v8::platform::InProcessStackDumping::kDisabled);

    v8::V8::InitializePlatform(g_platform.get());
    v8::V8::Initialize();
}

static void shutdown_platform(void) {
    v8::V8::Dispose();
    v8::V8::DisposePlatform();
    g_platform.reset();
}

void klyron_v8_init(const klyron_v8_config_t* config) {
    std::lock_guard<std::mutex> lock(g_mutex);
    if (g_initialized) return;

    init_config(config);
    init_platform();
    create_global_array_buffer_allocator(&g_config);

    g_initialized = true;
}

void klyron_v8_shutdown(void) {
    std::lock_guard<std::mutex> lock(g_mutex);
    if (!g_initialized) return;

    destroy_global_array_buffer_allocator();
    shutdown_platform();
    g_initialized = false;
}

bool klyron_v8_is_initialized(void) {
    return g_initialized;
}
