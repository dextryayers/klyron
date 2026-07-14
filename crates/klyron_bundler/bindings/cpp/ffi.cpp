#include "ffi.hpp"
#include <cstdint>

extern "C" {
    uint32_t klyron_bundler_version(void) { return 1; }
    int klyron_bundler_init(void) { return 0; }
    void klyron_bundler_cleanup(void) { }
}
