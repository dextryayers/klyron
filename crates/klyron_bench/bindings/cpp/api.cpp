#include "api.hpp"

namespace klyron_bench {

BenchClient::BenchClient() : config_() {}
BenchClient::BenchClient(const BenchConfig& config) : config_(config) {}
std::string BenchClient::version() const { return "1.0.0"; }
BenchConfig BenchClient::config() const { return config_; }

} // namespace
