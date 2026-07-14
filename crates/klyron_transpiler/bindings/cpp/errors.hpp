#pragma once
#include <stdexcept>
#include <string>

namespace klyron_transpiler {

class TranspilerException : public std::runtime_error {
public:
    explicit TranspilerException(const std::string& msg) : std::runtime_error(msg) {}
};

} // namespace
