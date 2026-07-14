#ifndef KLYRON_POSTGRES_BINDINGS_API_HPP
#define KLYRON_POSTGRES_BINDINGS_API_HPP

#include "types.hpp"
#include "config.hpp"

namespace klyron {

class Klyron::PostgresApi {
public:
  Klyron::PostgresApi();
  ~Klyron::PostgresApi();

  Klyron::PostgresResult process(const std::string& input);
  std::string version() const;
  bool ping();

private:
  class Impl;
  std::unique_ptr<Impl> impl_;
};

} // namespace klyron

#endif
