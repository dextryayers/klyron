#include "klyron_klyron_crypto_api.hpp"
#include <string>

namespace klyron {

CryptoProviderApi::CryptoProviderApi() {}

std::string CryptoProviderApi::version() const {
    return "klyron_crypto 0.1.0";
}

bool CryptoProviderApi::ping() {
    return true;
}

}
