#pragma once
#include <stdexcept>
#include <string>

namespace klyron::_shell {

class ShellError : public std::runtime_error {
public:
    explicit ShellError(const std::string& msg)
        : std::runtime_error("[Shell] " + msg) {}
};

class InitError : public ShellError {
public:
    explicit InitError(const std::string& msg) : ShellError(msg) {}
};

class OperationError : public ShellError {
public:
    explicit OperationError(const std::string& msg) : ShellError(msg) {}
};

}
