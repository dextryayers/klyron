#pragma once
#include <stdexcept>
#include <string>

namespace klyron::_deploy {

class DeployError : public std::runtime_error {
public:
    explicit DeployError(const std::string& msg)
        : std::runtime_error("[Deploy] " + msg) {}
};

class InitError : public DeployError {
public:
    explicit InitError(const std::string& msg) : DeployError(msg) {}
};

class OperationError : public DeployError {
public:
    explicit OperationError(const std::string& msg) : DeployError(msg) {}
};

}
