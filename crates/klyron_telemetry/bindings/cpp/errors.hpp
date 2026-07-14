#pragma once
#include <stdexcept>
#include <string>

namespace klyron::_telemetry {

class TelemetryError : public std::runtime_error {
public:
    explicit TelemetryError(const std::string& msg)
        : std::runtime_error("[Telemetry] " + msg) {}
};

class InitError : public TelemetryError {
public:
    explicit InitError(const std::string& msg) : TelemetryError(msg) {}
};

class OperationError : public TelemetryError {
public:
    explicit OperationError(const std::string& msg) : TelemetryError(msg) {}
};

}
