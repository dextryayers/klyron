#ifndef KLYRON_RUNTIME_BINDINGS_API_HPP
#define KLYRON_RUNTIME_BINDINGS_API_HPP

#include "types.hpp"
#include "config.hpp"

namespace klyron {

class Klyron::RuntimeApi {
public:
  Klyron::RuntimeApi();
  ~Klyron::RuntimeApi();

  Klyron::RuntimeResult process(const std::string& input);
  std::string version() const;
  bool ping();

private:
  class Impl;
  std::unique_ptr<Impl> impl_;
};

} // namespace klyron

#endif
