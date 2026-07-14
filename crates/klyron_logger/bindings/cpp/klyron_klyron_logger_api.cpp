#include "klyron_klyron_logger_api.hpp"
#include <string>

namespace klyron {

LoggerApi::LoggerApi() {}

std::string LoggerApi::version() const {
    return "klyron_logger 0.1.0";
}

bool LoggerApi::ping() {
    return true;
}

}
