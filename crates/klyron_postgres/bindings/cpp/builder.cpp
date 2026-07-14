#include "builder.hpp"

namespace klyron {

class Klyron::PostgresBuilder::Impl {
public:
  Klyron::PostgresConfig config;
  bool verbose = false;
};

Klyron::PostgresBuilder::Klyron::PostgresBuilder()
  : impl_(std::make_unique<Impl>()) {
}

Klyron::PostgresBuilder& Klyron::PostgresBuilder::withConfig(const Klyron::PostgresConfig& config) {
  impl_->config = config;
  return *this;
}

Klyron::PostgresBuilder& Klyron::PostgresBuilder::verbose(bool v) {
  impl_->verbose = v;
  return *this;
}

Klyron::PostgresBuilder::Klyron::PostgresInstance Klyron::PostgresBuilder::build() {
  Klyron::PostgresInstance inst;
  inst.config = impl_->config;
  inst.verbose = impl_->verbose;
  return inst;
}

} // namespace klyron
