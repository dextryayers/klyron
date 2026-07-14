#include "ffi.hpp"
#include "types.hpp"

extern "C" {
    void* config_create_config() {
        auto* cfg = new klyron::_config::ConfigConfig();
        return static_cast<void*>(cfg);
    }

    void config_free_config(void* ptr) {
        delete static_cast<klyron::_config::ConfigConfig*>(ptr);
    }
}
