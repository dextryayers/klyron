#pragma once
#include "types.hpp"

namespace klyron::_telemetry {

class TelemetryApi {
public:
    TelemetryApi();
    void execute();
    static std::string version();
};

}
