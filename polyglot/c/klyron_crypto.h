#ifndef KLYRON_CRYPTO_H
#define KLYRON_CRYPTO_H

#include "klyron_types.h"
#include <stdint.h>
#include <stddef.h>

klyron_string_t klyron_crypto_sha256(const char *data);
klyron_string_t klyron_crypto_sha512(const char *data);
void klyron_crypto_random_bytes(uint8_t *buf, size_t len);
void klyron_crypto_uuid4(char *out);

#endif
