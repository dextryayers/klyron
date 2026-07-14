#include "klyron_klyron_web_api.hpp"
#include <string>

namespace klyron {

WebApiApi::WebApiApi() {}

std::string WebApiApi::version() const {
    return "klyron_web 0.1.0";
}

bool WebApiApi::ping() {
    return true;
}

}
