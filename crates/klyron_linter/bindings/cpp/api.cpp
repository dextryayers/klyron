#include "api.hpp"

namespace klyron_linter {

LinterClient::LinterClient() : config_() {}
LinterClient::LinterClient(const LinterConfig& config) : config_(config) {}
std::string LinterClient::version() const { return "1.0.0"; }
LinterConfig LinterClient::config() const { return config_; }

} // namespace
