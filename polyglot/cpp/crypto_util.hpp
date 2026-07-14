#ifndef KLYRON_CRYPTO_UTIL_HPP
#define KLYRON_CRYPTO_UTIL_HPP

#include "klyron.hpp"
#include <openssl/evp.h>
#include <openssl/rand.h>
#include <openssl/hmac.h>
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

    static String hmac_sha256(const String &key, const String &data) {
        unsigned char result[32];
        unsigned int len = 32;
        HMAC(EVP_sha256(), key.data(), key.size(),
             (const unsigned char *)data.data(), data.size(),
             result, &len);
        return hex_encode(result, 32);
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

    static String base64_encode(const String &data) {
        BIO *bio = BIO_new(BIO_s_mem());
        BIO *b64 = BIO_new(BIO_f_base64());
        BIO_push(b64, bio);
        BIO_set_flags(b64, BIO_FLAGS_BASE64_NO_NL);
        BIO_write(b64, data.data(), data.size());
        BIO_flush(b64);
        char *encoded;
        long len = BIO_get_mem_data(bio, &encoded);
        String result(encoded, len);
        BIO_free_all(b64);
        return result;
    }

    static String base64_decode(const String &data) {
        BIO *bio = BIO_new_mem_buf(data.data(), data.size());
        BIO *b64 = BIO_new(BIO_f_base64());
        BIO_set_flags(b64, BIO_FLAGS_BASE64_NO_NL);
        BIO_push(b64, bio);
        Vec<char> decoded(data.size());
        int len = BIO_read(b64, decoded.data(), data.size());
        BIO_free_all(b64);
        if (len < 0) return "";
        return String(decoded.data(), len);
    }

    static String hex_decode(const String &hex) {
        if (hex.size() % 2 != 0) return "";
        String result;
        result.reserve(hex.size() / 2);
        for (size_t i = 0; i < hex.size(); i += 2) {
            unsigned int byte;
            std::istringstream(hex.substr(i, 2)) >> std::hex >> byte;
            result += static_cast<char>(byte);
        }
        return result;
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
