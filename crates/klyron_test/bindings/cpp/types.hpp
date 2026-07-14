#pragma once
#include <string>
#include <vector>

namespace klyron_test {

struct TestConfig {
    bool enabled = true;
    bool verbose = false;
};

struct TestResult {
    bool success = false;
    std::string message;
};

} // namespace
