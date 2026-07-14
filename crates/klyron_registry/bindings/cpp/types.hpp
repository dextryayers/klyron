#pragma once
#include <string>
#include <vector>

namespace klyron_registry {

struct RegistryConfig {
    bool enabled = true;
    bool verbose = false;
};

struct RegistryResult {
    bool success = false;
    std::string message;
};

} // namespace
