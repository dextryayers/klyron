#include "klyron_klyron_web_errors.h"

const char* klyron_klyron_web_error_string(klyron_klyron_web_error_t err) {
    switch(err) {
        case KLYRON_KLYRON_WEB_OK: return "ok";
        case KLYRON_KLYRON_WEB_ERR_GENERIC: return "generic error";
        default: return "unknown";
    }
}
