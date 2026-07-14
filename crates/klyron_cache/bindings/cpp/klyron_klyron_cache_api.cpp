#include "klyron_klyron_cache_api.hpp"
#include <string>

namespace klyron {

CacheManagerApi::CacheManagerApi() {}

std::string CacheManagerApi::version() const {
    return "klyron_cache 0.1.0";
}

bool CacheManagerApi::ping() {
    return true;
}

}
