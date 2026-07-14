#pragma once
#include "types.hpp"
#include "config.hpp"

namespace klyron::_telemetry {

class TelemetryBuilder {
public:
    TelemetryBuilder();
    TelemetryBuilder& set_config(const TelemetryConfig& cfg);
    TelemetryConfig config() const;
private:
    TelemetryConfig config_;
};

}
