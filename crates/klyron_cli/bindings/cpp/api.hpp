#ifndef KLYRON_CLI_BINDINGS_API_HPP
#define KLYRON_CLI_BINDINGS_API_HPP

#include "types.hpp"
#include "config.hpp"

namespace klyron {

class Klyron::CliApi {
public:
  Klyron::CliApi();
  ~Klyron::CliApi();

  Klyron::CliResult process(const std::string& input);
  std::string version() const;
  bool ping();

private:
  class Impl;
  std::unique_ptr<Impl> impl_;
};

} // namespace klyron

#endif
