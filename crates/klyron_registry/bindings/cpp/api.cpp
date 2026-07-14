#include "api.hpp"

namespace klyron_registry {

RegistryClient::RegistryClient() : config_() {}
RegistryClient::RegistryClient(const RegistryConfig& config) : config_(config) {}
std::string RegistryClient::version() const { return "1.0.0"; }
RegistryConfig RegistryClient::config() const { return config_; }

} // namespace
