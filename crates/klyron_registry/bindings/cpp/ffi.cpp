#include "ffi.hpp"
#include <cstdint>

extern "C" {
    uint32_t klyron_registry_version(void) { return 1; }
    int klyron_registry_init(void) { return 0; }
    void klyron_registry_cleanup(void) { }
}
