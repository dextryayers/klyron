#include "adapter_errors.h"

const char* klyron_adapter_error_string(int err) {
    switch (err) {
        case KLYRON_ADAPTER_OK: return "ok";
        case KLYRON_ADAPTER_ERR_INIT: return "init failed";
        case KLYRON_ADAPTER_ERR_OPERATION: return "operation failed";
        default: return "unknown error";
    }
}
