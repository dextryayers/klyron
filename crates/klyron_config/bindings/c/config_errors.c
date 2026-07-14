#include "config_errors.h"

const char* klyron_config_error_string(int err) {
    switch (err) {
        case KLYRON_CONFIG_OK: return "ok";
        case KLYRON_CONFIG_ERR_INIT: return "init failed";
        case KLYRON_CONFIG_ERR_OPERATION: return "operation failed";
        default: return "unknown error";
    }
}
