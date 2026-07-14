#include "klyron_klyron_process_errors.hpp"
#include <string>

namespace klyron {

std::string ProcessManagerError::error_string() const {
    return what();
}

}
