#ifndef KLYRON_CLI_BINDINGS_CONFIG_HPP
#define KLYRON_CLI_BINDINGS_CONFIG_HPP

#include "types.hpp"
#include <string>

namespace klyron {

struct Klyron::CliSettings {
  int maxRetries = 3;
  long timeoutMs = 5000;
  std::string logLevel = "info";
};

Klyron::CliConfig loadConfig(const std::string& path = "");

} // namespace klyron

#endif
