#ifndef KLYRON_SQLITE_BINDINGS_CONFIG_HPP
#define KLYRON_SQLITE_BINDINGS_CONFIG_HPP

#include "types.hpp"
#include <string>

namespace klyron {

struct Klyron::SqliteSettings {
  int maxRetries = 3;
  long timeoutMs = 5000;
  std::string logLevel = "info";
};

Klyron::SqliteConfig loadConfig(const std::string& path = "");

} // namespace klyron

#endif
