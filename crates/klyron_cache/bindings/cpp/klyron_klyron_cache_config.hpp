#ifndef KLYRON_KLYRON_CACHE_CONFIG_HPP
#define KLYRON_KLYRON_CACHE_CONFIG_HPP

#include <cstdint>

namespace klyron {
struct CacheManagerConfig {
    uint64_t timeout_ms;
};
}

#endif
