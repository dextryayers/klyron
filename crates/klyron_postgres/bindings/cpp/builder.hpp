#ifndef KLYRON_POSTGRES_BINDINGS_BUILDER_HPP
#define KLYRON_POSTGRES_BINDINGS_BUILDER_HPP

#include "types.hpp"
#include "config.hpp"
#include <memory>

namespace klyron {

class Klyron::PostgresBuilder {
public:
  Klyron::PostgresBuilder();
  Klyron::PostgresBuilder& withConfig(const Klyron::PostgresConfig& config);
  Klyron::PostgresBuilder& verbose(bool v);
  class Klyron::PostgresInstance build();

private:
  std::unique_ptr<class Impl> impl_;
};

class Klyron::PostgresBuilder::Klyron::PostgresInstance {
public:
  Klyron::PostgresConfig config;
  bool verbose = false;
};

} // namespace klyron

#endif
