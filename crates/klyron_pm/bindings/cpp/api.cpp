#include "api.hpp"

namespace klyron_pm {

PmClient::PmClient() : config_() {}
PmClient::PmClient(const PmConfig& config) : config_(config) {}
std::string PmClient::version() const { return "1.0.0"; }
PmConfig PmClient::config() const { return config_; }

} // namespace
