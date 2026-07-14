#ifndef KLYRON_UPDATER_BINDINGS_BUILDER_HPP
#define KLYRON_UPDATER_BINDINGS_BUILDER_HPP

#include "types.hpp"
#include "config.hpp"
#include <memory>

namespace klyron {

class Klyron::UpdaterBuilder {
public:
  Klyron::UpdaterBuilder();
  Klyron::UpdaterBuilder& withConfig(const Klyron::UpdaterConfig& config);
  Klyron::UpdaterBuilder& verbose(bool v);
  class Klyron::UpdaterInstance build();

private:
  std::unique_ptr<class Impl> impl_;
};

class Klyron::UpdaterBuilder::Klyron::UpdaterInstance {
public:
  Klyron::UpdaterConfig config;
  bool verbose = false;
};

} // namespace klyron

#endif
