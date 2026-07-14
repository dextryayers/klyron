#include "plugin_errors.h"

const char* klyron_plugin_error_string(int err) {
    switch (err) {
        case KLYRON_PLUGIN_OK: return "ok";
        case KLYRON_PLUGIN_ERR_INIT: return "init failed";
        case KLYRON_PLUGIN_ERR_OPERATION: return "operation failed";
        default: return "unknown error";
    }
}
