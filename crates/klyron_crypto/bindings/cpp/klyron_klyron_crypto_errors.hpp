#ifndef KLYRON_KLYRON_CRYPTO_ERRORS_HPP
#define KLYRON_KLYRON_CRYPTO_ERRORS_HPP

#include <stdexcept>
#include <string>

namespace klyron {
class CryptoProviderError : public std::runtime_error {
public:
    explicit CryptoProviderError(const std::string& msg) : std::runtime_error(msg) {}
};
}

#endif
