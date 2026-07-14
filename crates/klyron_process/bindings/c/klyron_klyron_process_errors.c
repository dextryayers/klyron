#include "klyron_klyron_process_errors.h"

const char* klyron_klyron_process_error_string(klyron_klyron_process_error_t err) {
    switch(err) {
        case KLYRON_KLYRON_PROCESS_OK: return "ok";
        case KLYRON_KLYRON_PROCESS_ERR_GENERIC: return "generic error";
        default: return "unknown";
    }
}
