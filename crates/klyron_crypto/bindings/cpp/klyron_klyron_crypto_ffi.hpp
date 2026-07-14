#ifndef KLYRON_KLYRON_CRYPTO_FFI_HPP
#define KLYRON_KLYRON_CRYPTO_FFI_HPP

extern "C" {
    const char* klyron_crypto_version();
}

inline const char* klyron_crypto_version_str() { return klyron_crypto_version(); }

#endif
