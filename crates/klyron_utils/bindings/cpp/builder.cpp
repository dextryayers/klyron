#include "builder.hpp"

namespace klyron {

class Klyron::UtilsBuilder::Impl {
public:
  Klyron::UtilsConfig config;
  bool verbose = false;
};

Klyron::UtilsBuilder::Klyron::UtilsBuilder()
  : impl_(std::make_unique<Impl>()) {
}

Klyron::UtilsBuilder& Klyron::UtilsBuilder::withConfig(const Klyron::UtilsConfig& config) {
  impl_->config = config;
  return *this;
}

Klyron::UtilsBuilder& Klyron::UtilsBuilder::verbose(bool v) {
  impl_->verbose = v;
  return *this;
}

Klyron::UtilsBuilder::Klyron::UtilsInstance Klyron::UtilsBuilder::build() {
  Klyron::UtilsInstance inst;
  inst.config = impl_->config;
  inst.verbose = impl_->verbose;
  return inst;
}

} // namespace klyron
