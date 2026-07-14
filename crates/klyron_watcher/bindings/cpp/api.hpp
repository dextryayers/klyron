#pragma once
#include "types.hpp"
#include <memory>

namespace klyron_watcher {

class WatcherClient {
public:
    WatcherClient();
    explicit WatcherClient(const WatcherConfig& config);
    std::string version() const;
    WatcherConfig config() const;

private:
    WatcherConfig config_;
};

} // namespace
