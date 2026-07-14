#include "builder.hpp"

namespace klyron_pm {
PmBuilder::PmBuilder() : config_() {}
PmBuilder& PmBuilder::enabled(bool v) { config_.enabled = v; return *this; }
PmBuilder& PmBuilder::verbose(bool v) { config_.verbose = v; return *this; }
PmConfig PmBuilder::build() { return config_; }
}
