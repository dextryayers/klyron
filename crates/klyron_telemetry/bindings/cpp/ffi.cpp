#include "ffi.hpp"
#include "types.hpp"

extern "C" {
    void* telemetry_create_config() {
        auto* cfg = new klyron::_telemetry::TelemetryConfig();
        return static_cast<void*>(cfg);
    }

    void telemetry_free_config(void* ptr) {
        delete static_cast<klyron::_telemetry::TelemetryConfig*>(ptr);
    }
}
