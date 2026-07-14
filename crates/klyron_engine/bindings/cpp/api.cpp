#include "api.hpp"

namespace klyron {

class Klyron::EngineApi::Impl {
public:
  bool initialized = false;
};

Klyron::EngineApi::Klyron::EngineApi()
  : impl_(std::make_unique<Impl>()) {
}

Klyron::EngineApi::~Klyron::EngineApi() = default;

Klyron::EngineResult Klyron::EngineApi::process(const std::string& input) {
  Klyron::EngineResult result;
  result.success = true;
  result.data = "processed: " + input;
  return result;
}

std::string Klyron::EngineApi::version() const {
  return "klyron_engine 0.1.0";
}

bool Klyron::EngineApi::ping() {
  return true;
}

} // namespace klyron
