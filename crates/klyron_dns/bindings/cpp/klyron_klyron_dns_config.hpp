#ifndef KLYRON_KLYRON_DNS_CONFIG_HPP
#define KLYRON_KLYRON_DNS_CONFIG_HPP

#include <cstdint>

namespace klyron {
struct DnsResolverConfig {
    uint64_t timeout_ms;
};
}

#endif
