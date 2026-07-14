#ifndef KLYRON_UPDATER_BINDINGS_TYPES_HPP
#define KLYRON_UPDATER_BINDINGS_TYPES_HPP

#include <string>
#include <vector>
#include <memory>
#include <functional>

namespace klyron {

struct Klyron::UpdaterConfig {
  bool enabled = true;
};

struct Klyron::UpdaterResult {
  bool success = false;
  std::string data;
  std::string error;
};

} // namespace klyron

#endif
