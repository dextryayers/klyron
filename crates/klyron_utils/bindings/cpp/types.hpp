#ifndef KLYRON_UTILS_BINDINGS_TYPES_HPP
#define KLYRON_UTILS_BINDINGS_TYPES_HPP

#include <string>
#include <vector>
#include <memory>
#include <functional>

namespace klyron {

struct Klyron::UtilsConfig {
  bool enabled = true;
};

struct Klyron::UtilsResult {
  bool success = false;
  std::string data;
  std::string error;
};

} // namespace klyron

#endif
