#include "builder.hpp"

namespace klyron {

class Klyron::EngineBuilder::Impl {
public:
  Klyron::EngineConfig config;
  bool verbose = false;
};

Klyron::EngineBuilder::Klyron::EngineBuilder()
  : impl_(std::make_unique<Impl>()) {
}

Klyron::EngineBuilder& Klyron::EngineBuilder::withConfig(const Klyron::EngineConfig& config) {
  impl_->config = config;
  return *this;
}

Klyron::EngineBuilder& Klyron::EngineBuilder::verbose(bool v) {
  impl_->verbose = v;
  return *this;
}

Klyron::EngineBuilder::Klyron::EngineInstance Klyron::EngineBuilder::build() {
  Klyron::EngineInstance inst;
  inst.config = impl_->config;
  inst.verbose = impl_->verbose;
  return inst;
}

} // namespace klyron
