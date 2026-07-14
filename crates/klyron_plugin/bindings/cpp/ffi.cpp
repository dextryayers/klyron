#include "ffi.hpp"
#include "types.hpp"

extern "C" {
    void* plugin_create_config() {
        auto* cfg = new klyron::_plugin::PluginConfig();
        return static_cast<void*>(cfg);
    }

    void plugin_free_config(void* ptr) {
        delete static_cast<klyron::_plugin::PluginConfig*>(ptr);
    }
}
