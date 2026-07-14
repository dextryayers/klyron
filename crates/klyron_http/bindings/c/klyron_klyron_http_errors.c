#include "klyron_klyron_http_errors.h"

const char* klyron_klyron_http_error_string(klyron_klyron_http_error_t err) {
    switch(err) {
        case KLYRON_KLYRON_HTTP_OK: return "ok";
        case KLYRON_KLYRON_HTTP_ERR_GENERIC: return "generic error";
        default: return "unknown";
    }
}
