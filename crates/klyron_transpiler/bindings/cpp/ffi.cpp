#include "ffi.hpp"
#include <cstdint>

extern "C" {
    uint32_t klyron_transpiler_version(void) { return 1; }
    int klyron_transpiler_init(void) { return 0; }
    void klyron_transpiler_cleanup(void) { }
}
