#include "errors.h"

const char* klyron_pm_error_string(klyron_pm_error_t err) {
    switch (err) {
        case KLYRON_PM_OK: return "success";
        case KLYRON_PM_ERR_FAILED: return "operation failed";
        default: return "unknown error";
    }
}
