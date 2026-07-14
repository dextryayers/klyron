#pragma once
#include <stdexcept>
#include <string>

namespace klyron_bench {

class BenchException : public std::runtime_error {
public:
    explicit BenchException(const std::string& msg) : std::runtime_error(msg) {}
};

} // namespace
