#ifndef KLYRON_KLYRON_CRYPTO_ERRORS_H
#define KLYRON_KLYRON_CRYPTO_ERRORS_H

typedef enum { KLYRON_KLYRON_CRYPTO_OK = 0, KLYRON_KLYRON_CRYPTO_ERR_GENERIC = -1 } klyron_klyron_crypto_error_t;
const char* klyron_klyron_crypto_error_string(klyron_klyron_crypto_error_t err);

#endif
