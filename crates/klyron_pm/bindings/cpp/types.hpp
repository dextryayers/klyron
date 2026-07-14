#pragma once
#include <string>
#include <vector>

namespace klyron_pm {

struct PmConfig {
    bool enabled = true;
    bool verbose = false;
};

struct PmResult {
    bool success = false;
    std::string message;
};

} // namespace
