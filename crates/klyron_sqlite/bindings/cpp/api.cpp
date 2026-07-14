#include "api.hpp"

namespace klyron {

class Klyron::SqliteApi::Impl {
public:
  bool initialized = false;
};

Klyron::SqliteApi::Klyron::SqliteApi()
  : impl_(std::make_unique<Impl>()) {
}

Klyron::SqliteApi::~Klyron::SqliteApi() = default;

Klyron::SqliteResult Klyron::SqliteApi::process(const std::string& input) {
  Klyron::SqliteResult result;
  result.success = true;
  result.data = "processed: " + input;
  return result;
}

std::string Klyron::SqliteApi::version() const {
  return "klyron_sqlite 0.1.0";
}

bool Klyron::SqliteApi::ping() {
  return true;
}

} // namespace klyron
