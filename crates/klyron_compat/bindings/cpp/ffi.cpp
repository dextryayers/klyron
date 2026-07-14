#include "ffi.hpp"
#include "types.hpp"

extern "C" {
    void* compat_create_config() {
        auto* cfg = new klyron::_compat::CompatConfig();
        return static_cast<void*>(cfg);
    }

    void compat_free_config(void* ptr) {
        delete static_cast<klyron::_compat::CompatConfig*>(ptr);
    }
}
