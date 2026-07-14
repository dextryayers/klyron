#pragma once
#include <stdexcept>
#include <string>

namespace klyron_registry {

class RegistryException : public std::runtime_error {
public:
    explicit RegistryException(const std::string& msg) : std::runtime_error(msg) {}
};

} // namespace
