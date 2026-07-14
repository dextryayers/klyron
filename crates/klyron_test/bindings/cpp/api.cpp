#include "api.hpp"

namespace klyron_test {

TestClient::TestClient() : config_() {}
TestClient::TestClient(const TestConfig& config) : config_(config) {}
std::string TestClient::version() const { return "1.0.0"; }
TestConfig TestClient::config() const { return config_; }

} // namespace
