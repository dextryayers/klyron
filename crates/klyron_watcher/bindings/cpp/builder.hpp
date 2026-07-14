#pragma once
#include "types.hpp"

namespace klyron_watcher {

class WatcherBuilder {
public:
    WatcherBuilder();
    WatcherBuilder& enabled(bool v);
    WatcherBuilder& verbose(bool v);
    WatcherConfig build();
private:
    WatcherConfig config_;
};

} // namespace
