#pragma once
#include <string>
#include <vector>

namespace klyron_bench {

struct BenchConfig {
    bool enabled = true;
    bool verbose = false;
};

struct BenchResult {
    bool success = false;
    std::string message;
};

} // namespace
