#ifndef KLYRON_MYSQL_BINDINGS_API_HPP
#define KLYRON_MYSQL_BINDINGS_API_HPP

#include "types.hpp"
#include "config.hpp"

namespace klyron {

class Klyron::MysqlApi {
public:
  Klyron::MysqlApi();
  ~Klyron::MysqlApi();

  Klyron::MysqlResult process(const std::string& input);
  std::string version() const;
  bool ping();

private:
  class Impl;
  std::unique_ptr<Impl> impl_;
};

} // namespace klyron

#endif
