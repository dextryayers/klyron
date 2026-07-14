#include "errors.h"

const char* klyron_registry_error_string(klyron_registry_error_t err) {
    switch (err) {
        case KLYRON_REGISTRY_OK: return "success";
        case KLYRON_REGISTRY_ERR_FAILED: return "operation failed";
        default: return "unknown error";
    }
}
