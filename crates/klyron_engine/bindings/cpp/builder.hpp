#ifndef KLYRON_ENGINE_BINDINGS_BUILDER_HPP
#define KLYRON_ENGINE_BINDINGS_BUILDER_HPP

#include "types.hpp"
#include "config.hpp"
#include <memory>

namespace klyron {

class Klyron::EngineBuilder {
public:
  Klyron::EngineBuilder();
  Klyron::EngineBuilder& withConfig(const Klyron::EngineConfig& config);
  Klyron::EngineBuilder& verbose(bool v);
  class Klyron::EngineInstance build();

private:
  std::unique_ptr<class Impl> impl_;
};

class Klyron::EngineBuilder::Klyron::EngineInstance {
public:
  Klyron::EngineConfig config;
  bool verbose = false;
};

} // namespace klyron

#endif
