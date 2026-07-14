#include "builder.hpp"

namespace klyron_registry {
RegistryBuilder::RegistryBuilder() : config_() {}
RegistryBuilder& RegistryBuilder::enabled(bool v) { config_.enabled = v; return *this; }
RegistryBuilder& RegistryBuilder::verbose(bool v) { config_.verbose = v; return *this; }
RegistryConfig RegistryBuilder::build() { return config_; }
}
