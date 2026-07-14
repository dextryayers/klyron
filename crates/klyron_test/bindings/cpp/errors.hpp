#pragma once
#include <stdexcept>
#include <string>

namespace klyron_test {

class TestException : public std::runtime_error {
public:
    explicit TestException(const std::string& msg) : std::runtime_error(msg) {}
};

} // namespace
