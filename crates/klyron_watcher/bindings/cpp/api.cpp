#include "api.hpp"

namespace klyron_watcher {

WatcherClient::WatcherClient() : config_() {}
WatcherClient::WatcherClient(const WatcherConfig& config) : config_(config) {}
std::string WatcherClient::version() const { return "1.0.0"; }
WatcherConfig WatcherClient::config() const { return config_; }

} // namespace
