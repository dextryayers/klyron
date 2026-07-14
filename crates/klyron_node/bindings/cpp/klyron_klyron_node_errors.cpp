#include "klyron_klyron_node_errors.hpp"
#include <string>

namespace klyron {

std::string NodeGlobalsError::error_string() const {
    return what();
}

}
