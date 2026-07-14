#pragma once
#include <stdexcept>
#include <string>

namespace klyron::_compat {

class CompatError : public std::runtime_error {
public:
    explicit CompatError(const std::string& msg)
        : std::runtime_error("[Compat] " + msg) {}
};

class InitError : public CompatError {
public:
    explicit InitError(const std::string& msg) : CompatError(msg) {}
};

class OperationError : public CompatError {
public:
    explicit OperationError(const std::string& msg) : CompatError(msg) {}
};

}
