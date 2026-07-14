#pragma once
#include <stdexcept>
#include <string>

namespace klyron_formatter {

class FormatterException : public std::runtime_error {
public:
    explicit FormatterException(const std::string& msg) : std::runtime_error(msg) {}
};

} // namespace
