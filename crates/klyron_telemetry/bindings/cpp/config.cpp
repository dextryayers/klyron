#include "config.hpp"

namespace klyron::_telemetry {

TelemetryConfigBuilder& TelemetryConfigBuilder::with_version(const std::string& v) {
    config_.version = v;
    return *this;
}

TelemetryConfig TelemetryConfigBuilder::build() {
    return config_;
}

}
