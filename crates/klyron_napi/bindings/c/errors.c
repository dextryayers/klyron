#include "errors.h"

const char* klyron_napi_error_string(klyron_napi_error_t err) {
    switch (err) {
        case KLYRON_NAPI_OK: return "success";
        case KLYRON_NAPI_ERR_MODULE_NOT_FOUND: return "module not found";
        case KLYRON_NAPI_ERR_LOAD_FAILED: return "load failed";
        case KLYRON_NAPI_ERR_VERSION_MISMATCH: return "version mismatch";
        case KLYRON_NAPI_ERR_INVALID_ARGUMENT: return "invalid argument";
        default: return "unknown error";
    }
}
