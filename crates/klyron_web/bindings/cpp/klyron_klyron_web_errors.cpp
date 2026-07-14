#include "klyron_klyron_web_errors.hpp"
#include <string>

namespace klyron {

std::string WebApiError::error_string() const {
    return what();
}

}
