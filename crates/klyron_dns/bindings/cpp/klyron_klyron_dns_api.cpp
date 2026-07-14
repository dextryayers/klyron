#include "klyron_klyron_dns_api.hpp"
#include <string>

namespace klyron {

DnsResolverApi::DnsResolverApi() {}

std::string DnsResolverApi::version() const {
    return "klyron_dns 0.1.0";
}

bool DnsResolverApi::ping() {
    return true;
}

}
