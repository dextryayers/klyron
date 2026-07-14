#ifndef KLYRON_KLYRON_WEB_ERRORS_HPP
#define KLYRON_KLYRON_WEB_ERRORS_HPP

#include <stdexcept>
#include <string>

namespace klyron {
class WebApiError : public std::runtime_error {
public:
    explicit WebApiError(const std::string& msg) : std::runtime_error(msg) {}
};
}

#endif
