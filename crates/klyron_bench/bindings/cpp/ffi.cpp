#include "ffi.hpp"
#include <cstdint>

extern "C" {
    uint32_t klyron_bench_version(void) { return 1; }
    int klyron_bench_init(void) { return 0; }
    void klyron_bench_cleanup(void) { }
}
