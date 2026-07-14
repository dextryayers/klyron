#include "klyron_crypto.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <openssl/sha.h>
#include <openssl/rand.h>

static void hex_encode(const unsigned char *in, size_t in_len, char *out) {
    static const char hex[] = "0123456789abcdef";
    for (size_t i = 0; i < in_len; i++) {
        out[i * 2] = hex[(in[i] >> 4) & 0xf];
        out[i * 2 + 1] = hex[in[i] & 0xf];
    }
    out[in_len * 2] = '\0';
}

klyron_string_t klyron_crypto_sha256(const char *data) {
    klyron_string_t result = {NULL, 0, 0};
    unsigned char hash[SHA256_DIGEST_LENGTH];
    SHA256((const unsigned char *)data, strlen(data), hash);
    result.data = (char *)malloc(65);
    if (!result.data) return result;
    hex_encode(hash, SHA256_DIGEST_LENGTH, result.data);
    result.len = 64;
    result.cap = 65;
    return result;
}

klyron_string_t klyron_crypto_sha512(const char *data) {
    klyron_string_t result = {NULL, 0, 0};
    unsigned char hash[SHA512_DIGEST_LENGTH];
    SHA512((const unsigned char *)data, strlen(data), hash);
    result.data = (char *)malloc(129);
    if (!result.data) return result;
    hex_encode(hash, SHA512_DIGEST_LENGTH, result.data);
    result.len = 128;
    result.cap = 129;
    return result;
}

void klyron_crypto_random_bytes(uint8_t *buf, size_t len) {
    RAND_bytes(buf, (int)len);
}

void klyron_crypto_uuid4(char *out) {
    uint8_t bytes[16];
    RAND_bytes(bytes, 16);
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    sprintf(out, "%02x%02x%02x%02x-%02x%02x-%02x%02x-%02x%02x-%02x%02x%02x%02x%02x%02x",
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5], bytes[6], bytes[7],
            bytes[8], bytes[9], bytes[10], bytes[11],
            bytes[12], bytes[13], bytes[14], bytes[15]);
}
