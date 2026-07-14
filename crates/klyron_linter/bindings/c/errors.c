#include "errors.h"

const char* klyron_linter_error_string(klyron_linter_error_t err) {
    switch (err) {
        case KLYRON_LINTER_OK: return "success";
        case KLYRON_LINTER_ERR_FAILED: return "operation failed";
        default: return "unknown error";
    }
}
