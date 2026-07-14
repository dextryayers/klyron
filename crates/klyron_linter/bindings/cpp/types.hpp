#pragma once
#include <string>
#include <vector>

namespace klyron_linter {

struct LinterConfig {
    bool enabled = true;
    bool verbose = false;
};

struct LinterResult {
    bool success = false;
    std::string message;
};

} // namespace
