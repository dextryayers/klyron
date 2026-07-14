#include "klyron_klyron_logger_errors.hpp"
#include <string>

namespace klyron {

std::string LoggerError::error_string() const {
    return what();
}

}
