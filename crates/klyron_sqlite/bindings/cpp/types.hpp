#ifndef KLYRON_SQLITE_BINDINGS_TYPES_HPP
#define KLYRON_SQLITE_BINDINGS_TYPES_HPP

#include <string>
#include <vector>
#include <memory>
#include <functional>

namespace klyron {

struct Klyron::SqliteConfig {
  bool enabled = true;
};

struct Klyron::SqliteResult {
  bool success = false;
  std::string data;
  std::string error;
};

} // namespace klyron

#endif
