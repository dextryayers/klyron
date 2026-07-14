#pragma once
#include <string>
#include <vector>

namespace klyron_watcher {

struct WatcherConfig {
    bool enabled = true;
    bool verbose = false;
};

struct WatcherResult {
    bool success = false;
    std::string message;
};

} // namespace
