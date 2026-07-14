#pragma once
#include <stdexcept>
#include <string>

namespace klyron::_config {

class ConfigError : public std::runtime_error {
public:
    explicit ConfigError(const std::string& msg)
        : std::runtime_error("[Config] " + msg) {}
};

class InitError : public ConfigError {
public:
    explicit InitError(const std::string& msg) : ConfigError(msg) {}
};

class OperationError : public ConfigError {
public:
    explicit OperationError(const std::string& msg) : ConfigError(msg) {}
};

}
