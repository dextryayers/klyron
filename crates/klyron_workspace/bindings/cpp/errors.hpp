#pragma once
#include <stdexcept>
#include <string>

namespace klyron::_workspace {

class WorkspaceError : public std::runtime_error {
public:
    explicit WorkspaceError(const std::string& msg)
        : std::runtime_error("[Workspace] " + msg) {}
};

class InitError : public WorkspaceError {
public:
    explicit InitError(const std::string& msg) : WorkspaceError(msg) {}
};

class OperationError : public WorkspaceError {
public:
    explicit OperationError(const std::string& msg) : WorkspaceError(msg) {}
};

}
