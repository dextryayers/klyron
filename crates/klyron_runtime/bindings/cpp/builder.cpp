#include "builder.hpp"

namespace klyron {

class Klyron::RuntimeBuilder::Impl {
public:
  Klyron::RuntimeConfig config;
  bool verbose = false;
};

Klyron::RuntimeBuilder::Klyron::RuntimeBuilder()
  : impl_(std::make_unique<Impl>()) {
}

Klyron::RuntimeBuilder& Klyron::RuntimeBuilder::withConfig(const Klyron::RuntimeConfig& config) {
  impl_->config = config;
  return *this;
}

Klyron::RuntimeBuilder& Klyron::RuntimeBuilder::verbose(bool v) {
  impl_->verbose = v;
  return *this;
}

Klyron::RuntimeBuilder::Klyron::RuntimeInstance Klyron::RuntimeBuilder::build() {
  Klyron::RuntimeInstance inst;
  inst.config = impl_->config;
  inst.verbose = impl_->verbose;
  return inst;
}

} // namespace klyron
