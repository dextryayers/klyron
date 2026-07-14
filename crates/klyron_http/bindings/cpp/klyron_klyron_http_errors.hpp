#ifndef KLYRON_KLYRON_HTTP_ERRORS_HPP
#define KLYRON_KLYRON_HTTP_ERRORS_HPP

#include <stdexcept>
#include <string>

namespace klyron {
class HttpServerError : public std::runtime_error {
public:
    explicit HttpServerError(const std::string& msg) : std::runtime_error(msg) {}
};
}

#endif
