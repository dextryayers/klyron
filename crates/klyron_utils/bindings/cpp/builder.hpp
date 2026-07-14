#ifndef KLYRON_UTILS_BINDINGS_BUILDER_HPP
#define KLYRON_UTILS_BINDINGS_BUILDER_HPP

#include "types.hpp"
#include "config.hpp"
#include <memory>

namespace klyron {

class Klyron::UtilsBuilder {
public:
  Klyron::UtilsBuilder();
  Klyron::UtilsBuilder& withConfig(const Klyron::UtilsConfig& config);
  Klyron::UtilsBuilder& verbose(bool v);
  class Klyron::UtilsInstance build();

private:
  std::unique_ptr<class Impl> impl_;
};

class Klyron::UtilsBuilder::Klyron::UtilsInstance {
public:
  Klyron::UtilsConfig config;
  bool verbose = false;
};

} // namespace klyron

#endif
