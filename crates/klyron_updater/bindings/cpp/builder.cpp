#include "builder.hpp"

namespace klyron {

class Klyron::UpdaterBuilder::Impl {
public:
  Klyron::UpdaterConfig config;
  bool verbose = false;
};

Klyron::UpdaterBuilder::Klyron::UpdaterBuilder()
  : impl_(std::make_unique<Impl>()) {
}

Klyron::UpdaterBuilder& Klyron::UpdaterBuilder::withConfig(const Klyron::UpdaterConfig& config) {
  impl_->config = config;
  return *this;
}

Klyron::UpdaterBuilder& Klyron::UpdaterBuilder::verbose(bool v) {
  impl_->verbose = v;
  return *this;
}

Klyron::UpdaterBuilder::Klyron::UpdaterInstance Klyron::UpdaterBuilder::build() {
  Klyron::UpdaterInstance inst;
  inst.config = impl_->config;
  inst.verbose = impl_->verbose;
  return inst;
}

} // namespace klyron
