#pragma once
#include <stdexcept>
#include <string>

namespace klyron::_adapter {

class AdapterError : public std::runtime_error {
public:
    explicit AdapterError(const std::string& msg)
        : std::runtime_error("[Adapter] " + msg) {}
};

class InitError : public AdapterError {
public:
    explicit InitError(const std::string& msg) : AdapterError(msg) {}
};

class OperationError : public AdapterError {
public:
    explicit OperationError(const std::string& msg) : AdapterError(msg) {}
};

}
