#include "klyron_klyron_crypto_errors.h"

const char* klyron_klyron_crypto_error_string(klyron_klyron_crypto_error_t err) {
    switch(err) {
        case KLYRON_KLYRON_CRYPTO_OK: return "ok";
        case KLYRON_KLYRON_CRYPTO_ERR_GENERIC: return "generic error";
        default: return "unknown";
    }
}
