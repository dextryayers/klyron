#ifndef KLYRON_ENGINE_BINDINGS_CONFIG_HPP
#define KLYRON_ENGINE_BINDINGS_CONFIG_HPP

#include "types.hpp"
#include <string>

namespace klyron {

struct Klyron::EngineSettings {
  int maxRetries = 3;
  long timeoutMs = 5000;
  std::string logLevel = "info";
};

Klyron::EngineConfig loadConfig(const std::string& path = "");

} // namespace klyron

#endif
