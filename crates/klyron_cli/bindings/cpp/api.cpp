#include "api.hpp"

namespace klyron {

class Klyron::CliApi::Impl {
public:
  bool initialized = false;
};

Klyron::CliApi::Klyron::CliApi()
  : impl_(std::make_unique<Impl>()) {
}

Klyron::CliApi::~Klyron::CliApi() = default;

Klyron::CliResult Klyron::CliApi::process(const std::string& input) {
  Klyron::CliResult result;
  result.success = true;
  result.data = "processed: " + input;
  return result;
}

std::string Klyron::CliApi::version() const {
  return "klyron_cli 0.1.0";
}

bool Klyron::CliApi::ping() {
  return true;
}

} // namespace klyron
