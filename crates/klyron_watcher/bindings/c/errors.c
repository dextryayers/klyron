#include "errors.h"

const char* klyron_watcher_error_string(klyron_watcher_error_t err) {
    switch (err) {
        case KLYRON_WATCHER_OK: return "success";
        case KLYRON_WATCHER_ERR_FAILED: return "operation failed";
        default: return "unknown error";
    }
}
