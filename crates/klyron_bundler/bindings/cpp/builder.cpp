#include "builder.hpp"

namespace klyron_bundler {
BundlerBuilder::BundlerBuilder() : config_() {}
BundlerBuilder& BundlerBuilder::enabled(bool v) { config_.enabled = v; return *this; }
BundlerBuilder& BundlerBuilder::verbose(bool v) { config_.verbose = v; return *this; }
BundlerConfig BundlerBuilder::build() { return config_; }
}
