#include "klyron_klyron_dns_errors.h"

const char* klyron_klyron_dns_error_string(klyron_klyron_dns_error_t err) {
    switch(err) {
        case KLYRON_KLYRON_DNS_OK: return "ok";
        case KLYRON_KLYRON_DNS_ERR_GENERIC: return "generic error";
        default: return "unknown";
    }
}
