#ifndef KLYRON_CLI_BINDINGS_TYPES_HPP
#define KLYRON_CLI_BINDINGS_TYPES_HPP

#include <string>
#include <vector>
#include <memory>
#include <functional>

namespace klyron {

struct Klyron::CliConfig {
  bool enabled = true;
};

struct Klyron::CliResult {
  bool success = false;
  std::string data;
  std::string error;
};

} // namespace klyron

#endif
