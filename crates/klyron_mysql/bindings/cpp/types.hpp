#ifndef KLYRON_MYSQL_BINDINGS_TYPES_HPP
#define KLYRON_MYSQL_BINDINGS_TYPES_HPP

#include <string>
#include <vector>
#include <memory>
#include <functional>

namespace klyron {

struct Klyron::MysqlConfig {
  bool enabled = true;
};

struct Klyron::MysqlResult {
  bool success = false;
  std::string data;
  std::string error;
};

} // namespace klyron

#endif
