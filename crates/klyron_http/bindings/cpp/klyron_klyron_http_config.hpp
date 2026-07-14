#ifndef KLYRON_KLYRON_HTTP_CONFIG_HPP
#define KLYRON_KLYRON_HTTP_CONFIG_HPP

#include <cstdint>

namespace klyron {
struct HttpServerConfig {
    uint64_t timeout_ms;
};
}

#endif
