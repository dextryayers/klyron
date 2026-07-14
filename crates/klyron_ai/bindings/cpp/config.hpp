#ifndef KLYRON_AI_BINDINGS_CONFIG_HPP
#define KLYRON_AI_BINDINGS_CONFIG_HPP

#include "types.hpp"
#include <string>

namespace klyron {

struct Klyron::AiSettings {
  int maxRetries = 3;
  long timeoutMs = 5000;
  std::string logLevel = "info";
};

Klyron::AiConfig loadConfig(const std::string& path = "");

} // namespace klyron

#endif
