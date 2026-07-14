#pragma once
#include <string>
#include <vector>

namespace klyron_bundler {

struct BundlerConfig {
    bool enabled = true;
    bool verbose = false;
};

struct BundlerResult {
    bool success = false;
    std::string message;
};

} // namespace
