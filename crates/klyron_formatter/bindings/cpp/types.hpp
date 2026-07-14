#pragma once
#include <string>
#include <vector>

namespace klyron_formatter {

struct FormatterConfig {
    bool enabled = true;
    bool verbose = false;
};

struct FormatterResult {
    bool success = false;
    std::string message;
};

} // namespace
