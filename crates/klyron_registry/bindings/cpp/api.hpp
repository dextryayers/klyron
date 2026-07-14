#pragma once
#include "types.hpp"
#include <memory>

namespace klyron_registry {

class RegistryClient {
public:
    RegistryClient();
    explicit RegistryClient(const RegistryConfig& config);
    std::string version() const;
    RegistryConfig config() const;

private:
    RegistryConfig config_;
};

} // namespace
