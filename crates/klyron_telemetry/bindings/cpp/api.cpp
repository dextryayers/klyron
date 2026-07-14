#include "api.hpp"

namespace klyron::_telemetry {

TelemetryApi::TelemetryApi() {}

void TelemetryApi::execute() {
}

std::string TelemetryApi::version() {
    return "klyron_telemetry@0.1.0";
}

}
