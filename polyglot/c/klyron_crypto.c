#include "klyron_crypto.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <openssl/sha.h>
#include <openssl/rand.h>
#include <openssl/evp.h>

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

klyron_string_t klyron_crypto_hex_encode(const uint8_t *in, size_t in_len) {
    klyron_string_t result = {NULL, 0, 0};
    result.data = (char *)malloc(in_len * 2 + 1);
    if (!result.data) return result;
    hex_encode(in, in_len, result.data);
    result.len = in_len * 2;
    result.cap = in_len * 2 + 1;
    return result;
}

klyron_string_t klyron_crypto_hex_decode(const char *hex) {
    klyron_string_t result = {NULL, 0, 0};
    size_t hex_len = strlen(hex);
    if (hex_len % 2 != 0) return result;
    size_t out_len = hex_len / 2;
    result.data = (char *)malloc(out_len + 1);
    if (!result.data) return result;
    for (size_t i = 0; i < out_len; i++) {
        unsigned int byte;
        sscanf(hex + i * 2, "%2x", &byte);
        result.data[i] = (unsigned char)byte;
    }
    result.data[out_len] = '\0';
    result.len = out_len;
    result.cap = out_len + 1;
    return result;
}

static const char b64_table[] = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

klyron_string_t klyron_crypto_base64_encode(const uint8_t *in, size_t in_len) {
    klyron_string_t result = {NULL, 0, 0};
    size_t out_len = 4 * ((in_len + 2) / 3);
    result.data = (char *)malloc(out_len + 1);
    if (!result.data) return result;
    size_t i, j = 0;
    for (i = 0; i < in_len; i += 3) {
        unsigned int val = (unsigned int)in[i] << 16;
        if (i + 1 < in_len) val |= (unsigned int)in[i + 1] << 8;
        if (i + 2 < in_len) val |= in[i + 2];
        result.data[j++] = b64_table[(val >> 18) & 0x3f];
        result.data[j++] = b64_table[(val >> 12) & 0x3f];
        result.data[j++] = (i + 1 < in_len) ? b64_table[(val >> 6) & 0x3f] : '=';
        result.data[j++] = (i + 2 < in_len) ? b64_table[val & 0x3f] : '=';
    }
    result.data[j] = '\0';
    result.len = j;
    result.cap = j + 1;
    return result;
}

static int b64_index(char c) {
    if (c >= 'A' && c <= 'Z') return c - 'A';
    if (c >= 'a' && c <= 'z') return c - 'a' + 26;
    if (c >= '0' && c <= '9') return c - '0' + 52;
    if (c == '+') return 62;
    if (c == '/') return 63;
    return -1;
}

klyron_string_t klyron_crypto_base64_decode(const char *in) {
    klyron_string_t result = {NULL, 0, 0};
    size_t in_len = strlen(in);
    if (in_len % 4 != 0) return result;
    size_t out_len = in_len / 4 * 3;
    if (in[in_len - 1] == '=') out_len--;
    if (in[in_len - 2] == '=') out_len--;
    result.data = (char *)malloc(out_len + 1);
    if (!result.data) return result;
    size_t j = 0;
    for (size_t i = 0; i < in_len; i += 4) {
        int a = b64_index(in[i]);
        int b = b64_index(in[i + 1]);
        int c = b64_index(in[i + 2]);
        int d = b64_index(in[i + 3]);
        unsigned int val = (a << 18) | (b << 12) | ((c >= 0) ? (c << 6) : 0) | ((d >= 0) ? d : 0);
        result.data[j++] = (val >> 16) & 0xff;
        if (c >= 0) result.data[j++] = (val >> 8) & 0xff;
        if (d >= 0) result.data[j++] = val & 0xff;
    }
    result.data[j] = '\0';
    result.len = j;
    result.cap = j + 1;
    return result;
}
