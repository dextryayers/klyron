#include "builder.hpp"

namespace klyron {

class Klyron::MysqlBuilder::Impl {
public:
  Klyron::MysqlConfig config;
  bool verbose = false;
};

Klyron::MysqlBuilder::Klyron::MysqlBuilder()
  : impl_(std::make_unique<Impl>()) {
}

Klyron::MysqlBuilder& Klyron::MysqlBuilder::withConfig(const Klyron::MysqlConfig& config) {
  impl_->config = config;
  return *this;
}

Klyron::MysqlBuilder& Klyron::MysqlBuilder::verbose(bool v) {
  impl_->verbose = v;
  return *this;
}

Klyron::MysqlBuilder::Klyron::MysqlInstance Klyron::MysqlBuilder::build() {
  Klyron::MysqlInstance inst;
  inst.config = impl_->config;
  inst.verbose = impl_->verbose;
  return inst;
}

} // namespace klyron
