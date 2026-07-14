#include "klyron_klyron_loader_errors.h"

const char* klyron_klyron_loader_error_string(klyron_klyron_loader_error_t err) {
    switch(err) {
        case KLYRON_KLYRON_LOADER_OK: return "ok";
        case KLYRON_KLYRON_LOADER_ERR_GENERIC: return "generic error";
        default: return "unknown";
    }
}
