#pragma once
#include "types.hpp"

namespace klyron::_telemetry {

class TelemetryConfigBuilder {
public:
    TelemetryConfigBuilder& with_version(const std::string& v);
    TelemetryConfig build();
private:
    TelemetryConfig config_;
};

}
