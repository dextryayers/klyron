#ifndef KLYRON_KLYRON_DNS_ERRORS_HPP
#define KLYRON_KLYRON_DNS_ERRORS_HPP

#include <stdexcept>
#include <string>

namespace klyron {
class DnsResolverError : public std::runtime_error {
public:
    explicit DnsResolverError(const std::string& msg) : std::runtime_error(msg) {}
};
}

#endif
