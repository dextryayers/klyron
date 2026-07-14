#include "builder.hpp"

namespace klyron_formatter {
FormatterBuilder::FormatterBuilder() : config_() {}
FormatterBuilder& FormatterBuilder::enabled(bool v) { config_.enabled = v; return *this; }
FormatterBuilder& FormatterBuilder::verbose(bool v) { config_.verbose = v; return *this; }
FormatterConfig FormatterBuilder::build() { return config_; }
}
