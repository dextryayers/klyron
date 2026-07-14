#pragma once
#include <string>
#include <vector>

namespace klyron_transpiler {

struct TranspilerConfig {
    bool enabled = true;
    bool verbose = false;
};

struct TranspilerResult {
    bool success = false;
    std::string message;
};

} // namespace
