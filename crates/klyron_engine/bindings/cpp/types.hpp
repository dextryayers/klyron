#ifndef KLYRON_ENGINE_BINDINGS_TYPES_HPP
#define KLYRON_ENGINE_BINDINGS_TYPES_HPP

#include <string>
#include <vector>
#include <memory>
#include <functional>

namespace klyron {

struct Klyron::EngineConfig {
  bool enabled = true;
};

struct Klyron::EngineResult {
  bool success = false;
  std::string data;
  std::string error;
};

} // namespace klyron

#endif
