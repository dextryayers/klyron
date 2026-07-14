#include "klyron_klyron_dns_errors.hpp"
#include <string>

namespace klyron {

std::string DnsResolverError::error_string() const {
    return what();
}

}
