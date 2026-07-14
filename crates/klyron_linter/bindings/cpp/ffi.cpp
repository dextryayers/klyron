#include "ffi.hpp"
#include <cstdint>

extern "C" {
    uint32_t klyron_linter_version(void) { return 1; }
    int klyron_linter_init(void) { return 0; }
    void klyron_linter_cleanup(void) { }
}
