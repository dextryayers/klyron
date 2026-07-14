#include "compat_errors.h"

const char* klyron_compat_error_string(int err) {
    switch (err) {
        case KLYRON_COMPAT_OK: return "ok";
        case KLYRON_COMPAT_ERR_INIT: return "init failed";
        case KLYRON_COMPAT_ERR_OPERATION: return "operation failed";
        default: return "unknown error";
    }
}
