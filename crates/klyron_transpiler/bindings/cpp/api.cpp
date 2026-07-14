#include "api.hpp"

namespace klyron_transpiler {

TranspilerClient::TranspilerClient() : config_() {}
TranspilerClient::TranspilerClient(const TranspilerConfig& config) : config_(config) {}
std::string TranspilerClient::version() const { return "1.0.0"; }
TranspilerConfig TranspilerClient::config() const { return config_; }

} // namespace
