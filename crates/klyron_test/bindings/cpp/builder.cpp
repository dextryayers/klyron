#include "builder.hpp"

namespace klyron_test {
TestBuilder::TestBuilder() : config_() {}
TestBuilder& TestBuilder::enabled(bool v) { config_.enabled = v; return *this; }
TestBuilder& TestBuilder::verbose(bool v) { config_.verbose = v; return *this; }
TestConfig TestBuilder::build() { return config_; }
}
