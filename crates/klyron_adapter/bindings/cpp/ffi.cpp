#include "ffi.hpp"
#include "types.hpp"

extern "C" {
    void* adapter_create_config() {
        auto* cfg = new klyron::_adapter::AdapterConfig();
        return static_cast<void*>(cfg);
    }

    void adapter_free_config(void* ptr) {
        delete static_cast<klyron::_adapter::AdapterConfig*>(ptr);
    }
}
