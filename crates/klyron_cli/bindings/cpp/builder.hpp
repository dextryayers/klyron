#ifndef KLYRON_CLI_BINDINGS_BUILDER_HPP
#define KLYRON_CLI_BINDINGS_BUILDER_HPP

#include "types.hpp"
#include "config.hpp"
#include <memory>

namespace klyron {

class Klyron::CliBuilder {
public:
  Klyron::CliBuilder();
  Klyron::CliBuilder& withConfig(const Klyron::CliConfig& config);
  Klyron::CliBuilder& verbose(bool v);
  class Klyron::CliInstance build();

private:
  std::unique_ptr<class Impl> impl_;
};

class Klyron::CliBuilder::Klyron::CliInstance {
public:
  Klyron::CliConfig config;
  bool verbose = false;
};

} // namespace klyron

#endif
