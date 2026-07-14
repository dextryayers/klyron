#ifndef KLYRON_POSTGRES_BINDINGS_TYPES_HPP
#define KLYRON_POSTGRES_BINDINGS_TYPES_HPP

#include <string>
#include <vector>
#include <memory>
#include <functional>

namespace klyron {

struct Klyron::PostgresConfig {
  bool enabled = true;
};

struct Klyron::PostgresResult {
  bool success = false;
  std::string data;
  std::string error;
};

} // namespace klyron

#endif
