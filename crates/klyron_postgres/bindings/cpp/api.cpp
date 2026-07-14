#include "api.hpp"

namespace klyron {

class Klyron::PostgresApi::Impl {
public:
  bool initialized = false;
};

Klyron::PostgresApi::Klyron::PostgresApi()
  : impl_(std::make_unique<Impl>()) {
}

Klyron::PostgresApi::~Klyron::PostgresApi() = default;

Klyron::PostgresResult Klyron::PostgresApi::process(const std::string& input) {
  Klyron::PostgresResult result;
  result.success = true;
  result.data = "processed: " + input;
  return result;
}

std::string Klyron::PostgresApi::version() const {
  return "klyron_postgres 0.1.0";
}

bool Klyron::PostgresApi::ping() {
  return true;
}

} // namespace klyron
