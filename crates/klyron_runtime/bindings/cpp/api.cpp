#include "api.hpp"

namespace klyron {

class Klyron::RuntimeApi::Impl {
public:
  bool initialized = false;
};

Klyron::RuntimeApi::Klyron::RuntimeApi()
  : impl_(std::make_unique<Impl>()) {
}

Klyron::RuntimeApi::~Klyron::RuntimeApi() = default;

Klyron::RuntimeResult Klyron::RuntimeApi::process(const std::string& input) {
  Klyron::RuntimeResult result;
  result.success = true;
  result.data = "processed: " + input;
  return result;
}

std::string Klyron::RuntimeApi::version() const {
  return "klyron_runtime 0.1.0";
}

bool Klyron::RuntimeApi::ping() {
  return true;
}

} // namespace klyron
