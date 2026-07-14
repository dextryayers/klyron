#ifndef KLYRON_MYSQL_BINDINGS_CONFIG_HPP
#define KLYRON_MYSQL_BINDINGS_CONFIG_HPP

#include "types.hpp"
#include <string>

namespace klyron {

struct Klyron::MysqlSettings {
  int maxRetries = 3;
  long timeoutMs = 5000;
  std::string logLevel = "info";
};

Klyron::MysqlConfig loadConfig(const std::string& path = "");

} // namespace klyron

#endif
