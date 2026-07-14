#ifndef KLYRON_MYSQL_BINDINGS_BUILDER_HPP
#define KLYRON_MYSQL_BINDINGS_BUILDER_HPP

#include "types.hpp"
#include "config.hpp"
#include <memory>

namespace klyron {

class Klyron::MysqlBuilder {
public:
  Klyron::MysqlBuilder();
  Klyron::MysqlBuilder& withConfig(const Klyron::MysqlConfig& config);
  Klyron::MysqlBuilder& verbose(bool v);
  class Klyron::MysqlInstance build();

private:
  std::unique_ptr<class Impl> impl_;
};

class Klyron::MysqlBuilder::Klyron::MysqlInstance {
public:
  Klyron::MysqlConfig config;
  bool verbose = false;
};

} // namespace klyron

#endif
