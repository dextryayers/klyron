#ifndef KLYRON_UPDATER_BINDINGS_CONFIG_HPP
#define KLYRON_UPDATER_BINDINGS_CONFIG_HPP

#include "types.hpp"
#include <string>

namespace klyron {

struct Klyron::UpdaterSettings {
  int maxRetries = 3;
  long timeoutMs = 5000;
  std::string logLevel = "info";
};

Klyron::UpdaterConfig loadConfig(const std::string& path = "");

} // namespace klyron

#endif
