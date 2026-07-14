#ifndef KLYRON_AI_BINDINGS_TYPES_HPP
#define KLYRON_AI_BINDINGS_TYPES_HPP

#include <string>
#include <vector>
#include <memory>
#include <functional>

namespace klyron {

struct Klyron::AiConfig {
  bool enabled = true;
};

struct Klyron::AiResult {
  bool success = false;
  std::string data;
  std::string error;
};

} // namespace klyron

#endif
