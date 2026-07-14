#ifndef KLYRON_RUNTIME_BINDINGS_CONFIG_HPP
#define KLYRON_RUNTIME_BINDINGS_CONFIG_HPP

#include "types.hpp"
#include <string>

namespace klyron {

struct Klyron::RuntimeSettings {
  int maxRetries = 3;
  long timeoutMs = 5000;
  std::string logLevel = "info";
};

Klyron::RuntimeConfig loadConfig(const std::string& path = "");

} // namespace klyron

#endif
