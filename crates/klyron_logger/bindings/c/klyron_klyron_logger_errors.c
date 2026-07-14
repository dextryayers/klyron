#include "klyron_klyron_logger_errors.h"

const char* klyron_klyron_logger_error_string(klyron_klyron_logger_error_t err) {
    switch(err) {
        case KLYRON_KLYRON_LOGGER_OK: return "ok";
        case KLYRON_KLYRON_LOGGER_ERR_GENERIC: return "generic error";
        default: return "unknown";
    }
}
