#include "api.hpp"

namespace klyron {

class Klyron::MysqlApi::Impl {
public:
  bool initialized = false;
};

Klyron::MysqlApi::Klyron::MysqlApi()
  : impl_(std::make_unique<Impl>()) {
}

Klyron::MysqlApi::~Klyron::MysqlApi() = default;

Klyron::MysqlResult Klyron::MysqlApi::process(const std::string& input) {
  Klyron::MysqlResult result;
  result.success = true;
  result.data = "processed: " + input;
  return result;
}

std::string Klyron::MysqlApi::version() const {
  return "klyron_mysql 0.1.0";
}

bool Klyron::MysqlApi::ping() {
  return true;
}

} // namespace klyron
