#include "ffi.hpp"
#include <cstdint>

extern "C" {
    uint32_t klyron_pm_version(void) { return 1; }
    int klyron_pm_init(void) { return 0; }
    void klyron_pm_cleanup(void) { }
}
