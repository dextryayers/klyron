#include "builder.hpp"

namespace klyron_transpiler {
TranspilerBuilder::TranspilerBuilder() : config_() {}
TranspilerBuilder& TranspilerBuilder::enabled(bool v) { config_.enabled = v; return *this; }
TranspilerBuilder& TranspilerBuilder::verbose(bool v) { config_.verbose = v; return *this; }
TranspilerConfig TranspilerBuilder::build() { return config_; }
}
