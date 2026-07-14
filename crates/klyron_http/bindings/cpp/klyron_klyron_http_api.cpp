#include "klyron_klyron_http_api.hpp"
#include <string>

namespace klyron {

HttpServerApi::HttpServerApi() {}

std::string HttpServerApi::version() const {
    return "klyron_http 0.1.0";
}

bool HttpServerApi::ping() {
    return true;
}

}
