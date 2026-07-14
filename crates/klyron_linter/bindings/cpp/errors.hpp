#pragma once
#include <stdexcept>
#include <string>

namespace klyron_linter {

class LinterException : public std::runtime_error {
public:
    explicit LinterException(const std::string& msg) : std::runtime_error(msg) {}
};

} // namespace
