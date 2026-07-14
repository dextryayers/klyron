#pragma once
#include <stdexcept>
#include <string>

namespace klyron_pm {

class PmException : public std::runtime_error {
public:
    explicit PmException(const std::string& msg) : std::runtime_error(msg) {}
};

} // namespace
