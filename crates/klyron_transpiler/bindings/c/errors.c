#include "errors.h"

const char* klyron_transpiler_error_string(klyron_transpiler_error_t err) {
    switch (err) {
        case KLYRON_TRANSPILER_OK: return "success";
        case KLYRON_TRANSPILER_ERR_FAILED: return "operation failed";
        default: return "unknown error";
    }
}
