#ifndef KLYRON_UTILS_BINDINGS_CONFIG_HPP
#define KLYRON_UTILS_BINDINGS_CONFIG_HPP

#include "types.hpp"
#include <string>

namespace klyron {

struct Klyron::UtilsSettings {
  int maxRetries = 3;
  long timeoutMs = 5000;
  std::string logLevel = "info";
};

Klyron::UtilsConfig loadConfig(const std::string& path = "");

} // namespace klyron

#endif
