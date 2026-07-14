#include "klyron_klyron_loader_errors.hpp"
#include <string>

namespace klyron {

std::string ModuleLoaderError::error_string() const {
    return what();
}

}
