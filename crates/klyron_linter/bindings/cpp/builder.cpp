#include "builder.hpp"

namespace klyron_linter {
LinterBuilder::LinterBuilder() : config_() {}
LinterBuilder& LinterBuilder::enabled(bool v) { config_.enabled = v; return *this; }
LinterBuilder& LinterBuilder::verbose(bool v) { config_.verbose = v; return *this; }
LinterConfig LinterBuilder::build() { return config_; }
}
