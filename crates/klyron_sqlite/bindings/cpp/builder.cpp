#include "builder.hpp"

namespace klyron {

class Klyron::SqliteBuilder::Impl {
public:
  Klyron::SqliteConfig config;
  bool verbose = false;
};

Klyron::SqliteBuilder::Klyron::SqliteBuilder()
  : impl_(std::make_unique<Impl>()) {
}

Klyron::SqliteBuilder& Klyron::SqliteBuilder::withConfig(const Klyron::SqliteConfig& config) {
  impl_->config = config;
  return *this;
}

Klyron::SqliteBuilder& Klyron::SqliteBuilder::verbose(bool v) {
  impl_->verbose = v;
  return *this;
}

Klyron::SqliteBuilder::Klyron::SqliteInstance Klyron::SqliteBuilder::build() {
  Klyron::SqliteInstance inst;
  inst.config = impl_->config;
  inst.verbose = impl_->verbose;
  return inst;
}

} // namespace klyron
