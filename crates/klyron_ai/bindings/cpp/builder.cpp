#include "builder.hpp"

namespace klyron {

class Klyron::AiBuilder::Impl {
public:
  Klyron::AiConfig config;
  bool verbose = false;
};

Klyron::AiBuilder::Klyron::AiBuilder()
  : impl_(std::make_unique<Impl>()) {
}

Klyron::AiBuilder& Klyron::AiBuilder::withConfig(const Klyron::AiConfig& config) {
  impl_->config = config;
  return *this;
}

Klyron::AiBuilder& Klyron::AiBuilder::verbose(bool v) {
  impl_->verbose = v;
  return *this;
}

Klyron::AiBuilder::Klyron::AiInstance Klyron::AiBuilder::build() {
  Klyron::AiInstance inst;
  inst.config = impl_->config;
  inst.verbose = impl_->verbose;
  return inst;
}

} // namespace klyron
