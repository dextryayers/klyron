#ifndef KLYRON_POSTGRES_BINDINGS_CONFIG_HPP
#define KLYRON_POSTGRES_BINDINGS_CONFIG_HPP

#include "types.hpp"
#include <string>

namespace klyron {

struct Klyron::PostgresSettings {
  int maxRetries = 3;
  long timeoutMs = 5000;
  std::string logLevel = "info";
};

Klyron::PostgresConfig loadConfig(const std::string& path = "");

} // namespace klyron

#endif
