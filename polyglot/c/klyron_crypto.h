#ifndef KLYRON_CRYPTO_H
#define KLYRON_CRYPTO_H

#include "klyron_types.h"

char *klyron_crypto_sha256(const char *data);
void klyron_crypto_random_bytes(uint8_t *buf, size_t len);
void klyron_crypto_uuid4(char *out);

#endif
