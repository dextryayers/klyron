#pragma once
#include <stdexcept>
#include <string>

namespace klyron::_plugin {

class PluginError : public std::runtime_error {
public:
    explicit PluginError(const std::string& msg)
        : std::runtime_error("[Plugin] " + msg) {}
};

class InitError : public PluginError {
public:
    explicit InitError(const std::string& msg) : PluginError(msg) {}
};

class OperationError : public PluginError {
public:
    explicit OperationError(const std::string& msg) : PluginError(msg) {}
};

}
