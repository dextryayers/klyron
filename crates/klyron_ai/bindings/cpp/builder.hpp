#ifndef KLYRON_AI_BINDINGS_BUILDER_HPP
#define KLYRON_AI_BINDINGS_BUILDER_HPP

#include "types.hpp"
#include "config.hpp"
#include <memory>

namespace klyron {

class Klyron::AiBuilder {
public:
  Klyron::AiBuilder();
  Klyron::AiBuilder& withConfig(const Klyron::AiConfig& config);
  Klyron::AiBuilder& verbose(bool v);
  class Klyron::AiInstance build();

private:
  std::unique_ptr<class Impl> impl_;
};

class Klyron::AiBuilder::Klyron::AiInstance {
public:
  Klyron::AiConfig config;
  bool verbose = false;
};

} // namespace klyron

#endif
