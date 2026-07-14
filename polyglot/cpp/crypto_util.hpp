#ifndef KLYRON_CRYPTO_UTIL_HPP
#define KLYRON_CRYPTO_UTIL_HPP

#include "klyron.hpp"
#include <openssl/evp.h>
#include <openssl/rand.h>
#include <iomanip>

namespace klyron {

class Crypto {
public:
    static String sha256(const String &data) {
        unsigned char hash[32];
        EVP_MD_CTX *ctx = EVP_MD_CTX_new();
        EVP_DigestInit_ex(ctx, EVP_sha256(), nullptr);
        EVP_DigestUpdate(ctx, data.data(), data.size());
        EVP_DigestFinal_ex(ctx, hash, nullptr);
        EVP_MD_CTX_free(ctx);
        return hex_encode(hash, 32);
    }

    static String sha512(const String &data) {
        unsigned char hash[64];
        EVP_MD_CTX *ctx = EVP_MD_CTX_new();
        EVP_DigestInit_ex(ctx, EVP_sha512(), nullptr);
        EVP_DigestUpdate(ctx, data.data(), data.size());
        EVP_DigestFinal_ex(ctx, hash, nullptr);
        EVP_MD_CTX_free(ctx);
        return hex_encode(hash, 64);
    }

    static Vec<uint8_t> random_bytes(size_t len) {
        Vec<uint8_t> buf(len);
        RAND_bytes(buf.data(), static_cast<int>(len));
        return buf;
    }

    static String random_hex(size_t len) {
        auto bytes = random_bytes(len);
        return hex_encode(bytes.data(), bytes.size());
    }

    static String uuid4() {
        uint8_t bytes[16];
        RAND_bytes(bytes, 16);
        bytes[6] = (bytes[6] & 0x0f) | 0x40;
        bytes[8] = (bytes[8] & 0x3f) | 0x80;
        std::ostringstream os;
        os << std::hex << std::setfill('0');
        for (int i = 0; i < 16; i++) {
            os << std::setw(2) << static_cast<int>(bytes[i]);
            if (i == 3 || i == 5 || i == 7 || i == 9) os << '-';
        }
        return os.str();
    }

private:
    static String hex_encode(const unsigned char *data, size_t len) {
        std::ostringstream os;
        os << std::hex << std::setfill('0');
        for (size_t i = 0; i < len; i++)
            os << std::setw(2) << static_cast<int>(data[i]);
        return os.str();
    }
};

} // namespace klyron

#endif
