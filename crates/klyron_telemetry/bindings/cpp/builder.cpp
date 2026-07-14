#include "builder.hpp"

namespace klyron::_telemetry {

TelemetryBuilder::TelemetryBuilder() : config_{} {}

TelemetryBuilder& TelemetryBuilder::set_config(const TelemetryConfig& cfg) {
    config_ = cfg;
    return *this;
}

TelemetryConfig TelemetryBuilder::config() const {
    return config_;
}

}
