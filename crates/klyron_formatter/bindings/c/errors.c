#include "errors.h"

const char* klyron_formatter_error_string(klyron_formatter_error_t err) {
    switch (err) {
        case KLYRON_FORMATTER_OK: return "success";
        case KLYRON_FORMATTER_ERR_FAILED: return "operation failed";
        default: return "unknown error";
    }
}
