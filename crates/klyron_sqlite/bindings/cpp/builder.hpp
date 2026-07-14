#ifndef KLYRON_SQLITE_BINDINGS_BUILDER_HPP
#define KLYRON_SQLITE_BINDINGS_BUILDER_HPP

#include "types.hpp"
#include "config.hpp"
#include <memory>

namespace klyron {

class Klyron::SqliteBuilder {
public:
  Klyron::SqliteBuilder();
  Klyron::SqliteBuilder& withConfig(const Klyron::SqliteConfig& config);
  Klyron::SqliteBuilder& verbose(bool v);
  class Klyron::SqliteInstance build();

private:
  std::unique_ptr<class Impl> impl_;
};

class Klyron::SqliteBuilder::Klyron::SqliteInstance {
public:
  Klyron::SqliteConfig config;
  bool verbose = false;
};

} // namespace klyron

#endif
