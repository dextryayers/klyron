#pragma once
#include <stdexcept>
#include <string>

namespace klyron::_docker {

class DockerError : public std::runtime_error {
public:
    explicit DockerError(const std::string& msg)
        : std::runtime_error("[Docker] " + msg) {}
};

class InitError : public DockerError {
public:
    explicit InitError(const std::string& msg) : DockerError(msg) {}
};

class OperationError : public DockerError {
public:
    explicit OperationError(const std::string& msg) : DockerError(msg) {}
};

}
