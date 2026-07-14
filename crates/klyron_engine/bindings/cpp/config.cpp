#include "config.hpp"
#include <fstream>
#include <nlohmann/json.hpp>

namespace klyron {

Klyron::EngineConfig loadConfig(const std::string& path) {
  Klyron::EngineConfig config;
  if (!path.empty()) {
    std::ifstream file(path);
    if (file.is_open()) {
      try {
        auto json = nlohmann::json::parse(file);
        config.enabled = json.value("enabled", true);
      } catch (...) {}
    }
  }
  return config;
}

} // namespace klyron
