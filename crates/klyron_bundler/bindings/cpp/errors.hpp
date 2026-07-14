#pragma once
#include <stdexcept>
#include <string>

namespace klyron_bundler {

class BundlerException : public std::runtime_error {
public:
    explicit BundlerException(const std::string& msg) : std::runtime_error(msg) {}
};

} // namespace
