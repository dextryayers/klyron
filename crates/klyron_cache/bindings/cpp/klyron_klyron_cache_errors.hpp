#ifndef KLYRON_KLYRON_CACHE_ERRORS_HPP
#define KLYRON_KLYRON_CACHE_ERRORS_HPP

#include <stdexcept>
#include <string>

namespace klyron {
class CacheManagerError : public std::runtime_error {
public:
    explicit CacheManagerError(const std::string& msg) : std::runtime_error(msg) {}
};
}

#endif
