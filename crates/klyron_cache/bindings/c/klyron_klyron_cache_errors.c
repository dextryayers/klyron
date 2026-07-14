#include "klyron_klyron_cache_errors.h"

const char* klyron_klyron_cache_error_string(klyron_klyron_cache_error_t err) {
    switch(err) {
        case KLYRON_KLYRON_CACHE_OK: return "ok";
        case KLYRON_KLYRON_CACHE_ERR_GENERIC: return "generic error";
        default: return "unknown";
    }
}
