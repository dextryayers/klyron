#include "ffi.hpp"
#include <cstdint>

extern "C" {
    uint32_t klyron_formatter_version(void) { return 1; }
    int klyron_formatter_init(void) { return 0; }
    void klyron_formatter_cleanup(void) { }
}
