#include "builder.hpp"

namespace klyron_bench {
BenchBuilder::BenchBuilder() : config_() {}
BenchBuilder& BenchBuilder::enabled(bool v) { config_.enabled = v; return *this; }
BenchBuilder& BenchBuilder::verbose(bool v) { config_.verbose = v; return *this; }
BenchConfig BenchBuilder::build() { return config_; }
}
