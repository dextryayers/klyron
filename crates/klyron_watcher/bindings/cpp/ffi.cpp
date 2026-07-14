#include "ffi.hpp"
#include <cstdint>

extern "C" {
    uint32_t klyron_watcher_version(void) { return 1; }
    int klyron_watcher_init(void) { return 0; }
    void klyron_watcher_cleanup(void) { }
}
