#include "telemetry_errors.h"

const char* klyron_telemetry_error_string(int err) {
    switch (err) {
        case KLYRON_TELEMETRY_OK: return "ok";
        case KLYRON_TELEMETRY_ERR_INIT: return "init failed";
        case KLYRON_TELEMETRY_ERR_OPERATION: return "operation failed";
        default: return "unknown error";
    }
}
