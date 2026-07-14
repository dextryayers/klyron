#ifndef KLYRON_KLYRON_LOADER_ERRORS_HPP
#define KLYRON_KLYRON_LOADER_ERRORS_HPP

#include <stdexcept>
#include <string>

namespace klyron {
class ModuleResolverError : public std::runtime_error {
public:
    explicit ModuleResolverError(const std::string& msg) : std::runtime_error(msg) {}
};
}

#endif
