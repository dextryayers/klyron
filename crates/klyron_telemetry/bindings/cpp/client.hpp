#pragma once
#include "types.hpp"

namespace klyron::_telemetry {

class TelemetryClient {
public:
    explicit TelemetryClient(const TelemetryConfig& config);
    void execute();
private:
    TelemetryConfig config_;
};

}
