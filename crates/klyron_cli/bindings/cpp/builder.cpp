#include "builder.hpp"

namespace klyron {

class Klyron::CliBuilder::Impl {
public:
  Klyron::CliConfig config;
  bool verbose = false;
};

Klyron::CliBuilder::Klyron::CliBuilder()
  : impl_(std::make_unique<Impl>()) {
}

Klyron::CliBuilder& Klyron::CliBuilder::withConfig(const Klyron::CliConfig& config) {
  impl_->config = config;
  return *this;
}

Klyron::CliBuilder& Klyron::CliBuilder::verbose(bool v) {
  impl_->verbose = v;
  return *this;
}

Klyron::CliBuilder::Klyron::CliInstance Klyron::CliBuilder::build() {
  Klyron::CliInstance inst;
  inst.config = impl_->config;
  inst.verbose = impl_->verbose;
  return inst;
}

} // namespace klyron
