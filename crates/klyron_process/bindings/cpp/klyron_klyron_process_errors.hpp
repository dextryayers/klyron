#ifndef KLYRON_KLYRON_PROCESS_ERRORS_HPP
#define KLYRON_KLYRON_PROCESS_ERRORS_HPP

#include <stdexcept>
#include <string>

namespace klyron {
class ProcessManagerError : public std::runtime_error {
public:
    explicit ProcessManagerError(const std::string& msg) : std::runtime_error(msg) {}
};
}

#endif
