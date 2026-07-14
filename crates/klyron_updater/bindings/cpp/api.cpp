#include "api.hpp"

namespace klyron {

class Klyron::UpdaterApi::Impl {
public:
  bool initialized = false;
};

Klyron::UpdaterApi::Klyron::UpdaterApi()
  : impl_(std::make_unique<Impl>()) {
}

Klyron::UpdaterApi::~Klyron::UpdaterApi() = default;

Klyron::UpdaterResult Klyron::UpdaterApi::process(const std::string& input) {
  Klyron::UpdaterResult result;
  result.success = true;
  result.data = "processed: " + input;
  return result;
}

std::string Klyron::UpdaterApi::version() const {
  return "klyron_updater 0.1.0";
}

bool Klyron::UpdaterApi::ping() {
  return true;
}

} // namespace klyron
