#include "klyron_klyron_crypto_errors.hpp"
#include <string>

namespace klyron {

std::string CryptoProviderError::error_string() const {
    return what();
}

}
