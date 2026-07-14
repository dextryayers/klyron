#include "klyron_klyron_http_errors.hpp"
#include <string>

namespace klyron {

std::string HttpServerError::error_string() const {
    return what();
}

}
