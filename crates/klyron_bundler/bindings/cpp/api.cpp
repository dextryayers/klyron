#include "api.hpp"

namespace klyron_bundler {

BundlerClient::BundlerClient() : config_() {}
BundlerClient::BundlerClient(const BundlerConfig& config) : config_(config) {}
std::string BundlerClient::version() const { return "1.0.0"; }
BundlerConfig BundlerClient::config() const { return config_; }

} // namespace
