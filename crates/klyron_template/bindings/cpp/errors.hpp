#pragma once
#include <stdexcept>
#include <string>

namespace klyron::_template {

class TemplateError : public std::runtime_error {
public:
    explicit TemplateError(const std::string& msg)
        : std::runtime_error("[Template] " + msg) {}
};

class InitError : public TemplateError {
public:
    explicit InitError(const std::string& msg) : TemplateError(msg) {}
};

class OperationError : public TemplateError {
public:
    explicit OperationError(const std::string& msg) : TemplateError(msg) {}
};

}
