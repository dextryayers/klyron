#include "errors.h"

const char* klyron_bundler_error_string(klyron_bundler_error_t err) {
    switch (err) {
        case KLYRON_BUNDLER_OK: return "success";
        case KLYRON_BUNDLER_ERR_FAILED: return "operation failed";
        default: return "unknown error";
    }
}
