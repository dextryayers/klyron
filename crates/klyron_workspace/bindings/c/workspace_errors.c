#include "workspace_errors.h"

const char* klyron_workspace_error_string(int err) {
    switch (err) {
        case KLYRON_WORKSPACE_OK: return "ok";
        case KLYRON_WORKSPACE_ERR_INIT: return "init failed";
        case KLYRON_WORKSPACE_ERR_OPERATION: return "operation failed";
        default: return "unknown error";
    }
}
