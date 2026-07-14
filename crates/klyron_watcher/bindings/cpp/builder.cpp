#include "builder.hpp"

namespace klyron_watcher {
WatcherBuilder::WatcherBuilder() : config_() {}
WatcherBuilder& WatcherBuilder::enabled(bool v) { config_.enabled = v; return *this; }
WatcherBuilder& WatcherBuilder::verbose(bool v) { config_.verbose = v; return *this; }
WatcherConfig WatcherBuilder::build() { return config_; }
}
