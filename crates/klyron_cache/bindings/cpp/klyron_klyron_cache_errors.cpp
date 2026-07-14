#include "klyron_klyron_cache_errors.hpp"
#include <string>

namespace klyron {

std::string CacheManagerError::error_string() const {
    return what();
}

}
