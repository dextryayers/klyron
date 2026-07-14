#ifndef KLYRON_KLYRON_LOGGER_ERRORS_HPP
#define KLYRON_KLYRON_LOGGER_ERRORS_HPP

#include <stdexcept>
#include <string>

namespace klyron {
class LoggerError : public std::runtime_error {
public:
    explicit LoggerError(const std::string& msg) : std::runtime_error(msg) {}
};
}

#endif
