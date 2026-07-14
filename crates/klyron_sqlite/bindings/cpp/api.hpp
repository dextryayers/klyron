#ifndef KLYRON_SQLITE_BINDINGS_API_HPP
#define KLYRON_SQLITE_BINDINGS_API_HPP

#include "types.hpp"
#include "config.hpp"

namespace klyron {

class Klyron::SqliteApi {
public:
  Klyron::SqliteApi();
  ~Klyron::SqliteApi();

  Klyron::SqliteResult process(const std::string& input);
  std::string version() const;
  bool ping();

private:
  class Impl;
  std::unique_ptr<Impl> impl_;
};

} // namespace klyron

#endif
