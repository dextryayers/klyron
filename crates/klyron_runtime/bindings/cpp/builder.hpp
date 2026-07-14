#ifndef KLYRON_RUNTIME_BINDINGS_BUILDER_HPP
#define KLYRON_RUNTIME_BINDINGS_BUILDER_HPP

#include "types.hpp"
#include "config.hpp"
#include <memory>

namespace klyron {

class Klyron::RuntimeBuilder {
public:
  Klyron::RuntimeBuilder();
  Klyron::RuntimeBuilder& withConfig(const Klyron::RuntimeConfig& config);
  Klyron::RuntimeBuilder& verbose(bool v);
  class Klyron::RuntimeInstance build();

private:
  std::unique_ptr<class Impl> impl_;
};

class Klyron::RuntimeBuilder::Klyron::RuntimeInstance {
public:
  Klyron::RuntimeConfig config;
  bool verbose = false;
};

} // namespace klyron

#endif
